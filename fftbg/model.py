import logging
import pickle

import matplotlib.pyplot as plt
import numpy as np
from sklearn.compose import ColumnTransformer
from sklearn.metrics import precision_score, recall_score, accuracy_score
from sklearn.metrics import roc_curve, roc_auc_score
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, StandardScaler
from tensorflow import keras

import combatant
import config
import data
import tournament

LOG = logging.getLogger(__name__)


def split(xs, y, size):
    splits = train_test_split(*xs, y, test_size=size)
    train_y = splits[-2]
    test_y = splits[-1]
    train_X = [splits[i * 2] for i in range(8)]
    test_X = [splits[i * 2 + 1] for i in range(8)]
    return train_X, test_X, train_y, test_y


NUM_COLUMNS = combatant.NUMERIC + tournament.NUMERIC
CAT_COLUMNS = ['Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill',
               'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory', 'Map']


def get_skill_columns(df):
    return [c for c in df.keys() if combatant.SKILL_TAG in c]


def get_all_columns(df):
    return NUM_COLUMNS + CAT_COLUMNS + get_skill_columns(df)


def main():
    LOG.info('Going to compute tournament model')
    df = data.read_combatants()

    skill_columns = get_skill_columns(df)
    all_columns = get_all_columns(df)
    dfs = df[all_columns]
    dfs = dfs.sort_index(axis=1)

    combatant_dfs = [df[df['UIDX'] == i][all_columns] for i in range(8)]
    combatant_dfs = [df.sort_index(axis=1) for df in combatant_dfs]

    pipeline = ColumnTransformer([
        ('cat',
         OneHotEncoder(),
         CAT_COLUMNS),
        ('none',
         'passthrough',
         NUM_COLUMNS + skill_columns),
    ])

    LOG.info('Pre-processing data')
    pipeline.fit(dfs)
    combatant_dfs = [pipeline.transform(combatant_df).astype('float32')
                     for combatant_df in combatant_dfs]
    winner = df[df['UIDX'] == 0]['LeftWins']

    train_X, test_X, train_y, test_y = split(combatant_dfs, winner, size=0.3)
    test_X, valid_X, test_y, valid_y = split(test_X, test_y, size=0.2)

    scalers = [StandardScaler() for _ in range(len(train_X))]
    train_X = [scaler.fit_transform(train_xi) for (scaler, train_xi) in zip(scalers, train_X)]
    test_X = [scaler.transform(test_xi) for (scaler, test_xi) in zip(scalers, test_X)]
    valid_X = [scaler.transform(valid_xi) for (scaler, valid_xi) in zip(scalers, valid_X)]

    # Augment tests:
    # train_X2 = train_X[4:] + train_X[:4]
    # train_y2 = ~train_y
    #
    # train_X = [np.append(train_X[i], train_X2[i], axis=0) for i in range(8)]
    # train_y = np.append(train_y, train_y2)

    LOG.info(f'Training data shapes    X:{str(train_X[0].shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes     X:{str(test_X[0].shape):>14} y:{str(test_y.shape):>9}')
    LOG.info(f'Validation data shapes  X:{str(valid_X[0].shape):>14} y:{str(valid_y.shape):>9}')

    combatant_size = train_X[0].shape[1]
    layer_size = combatant_size

    early_stopping_cb, model = model_classic(combatant_size, layer_size)
    model.fit(train_X,
              train_y,
              epochs=100,
              verbose=1,
              validation_data=(valid_X, valid_y),
              callbacks=[early_stopping_cb])
    LOG.info('Done training model')

    if config.SAVE_MODEL:
        LOG.info(f'Saving model at {config.MODEL_PATH}')
        model.save(config.MODEL_PATH)
        LOG.info(f'Saving column transformation pipeline at {config.COLUMN_XFORM_PATH}')
        with config.COLUMN_XFORM_PATH.open(mode='wb') as f:
            pickle.dump(pipeline, f)
        LOG.info(f'Saving feature scalers at {config.FEATURE_SCALER_PATH}')
        with config.FEATURE_SCALER_PATH.open(mode='wb') as f:
            pickle.dump(scalers, f)

    train_y_scores = score_model(model, 'train', train_X, train_y)
    test_y_scores = score_model(model, 'test', test_X, test_y)

    fpr, tpr, thresholds = roc_curve(train_y, train_y_scores)
    plt.plot(fpr, tpr, linewidth=2, label='training')

    fpr, tpr, thresholds = roc_curve(test_y, test_y_scores)
    plt.plot(fpr, tpr, linewidth=2, c='c', label='test')
    plt.plot([0, 1], [0, 1], 'k--')
    plt.xlabel('False Positive Rate')
    plt.ylabel('True Positive Rate (Recall)')
    plt.grid()
    plt.legend()
    plt.show()


class MCDropout(keras.layers.Dropout):
    def call(self, inputs, _training=True):
        return super().call(inputs, training=True)


def dense_single(n):
    d = keras.layers.Dense(
        n,
        kernel_initializer='he_normal',
        kernel_regularizer=keras.regularizers.l2(0.01),
        use_bias=False)
    bn = keras.layers.BatchNormalization()
    a = keras.layers.Activation('elu')
    return lambda x: a(bn(d(x)))


def dense(n):
    d1 = dense_single(n)
    d2 = dense_single(n)
    d3 = dense_single(n)
    mc = MCDropout(0.25)
    return lambda x: mc(d3(d2(d1(x))))


def model_classic(combatant_size, layer_size):
    inputs = [keras.layers.Input(shape=(combatant_size,)) for _ in range(8)]
    combatant_layer = dense(layer_size // 10)
    combatant_nodes = [combatant_layer(c_input) for c_input in inputs]

    # ally_layer = dense(layer_size // 10)
    foe_layer = dense(layer_size // 100)
    # team_1_ally_layers = []
    team_1_foe_layers = []
    # team_2_ally_layers = []
    team_2_foe_layers = []

    for combatant_node in combatant_nodes[:4]:
        # for ally_node in combatant_nodes[:4]:
        #     if combatant_node is ally_node:
        #         continue
        #     team_1_ally_layers.append(ally_layer(keras.layers.concatenate([combatant_node, ally_node])))
        for foe_node in combatant_nodes[4:]:
            team_1_foe_layers.append(foe_layer(keras.layers.concatenate([combatant_node, foe_node])))

    for combatant_node in combatant_nodes[4:]:
        # for ally_node in combatant_nodes[4:]:
        #     if combatant_node is ally_node:
        #         continue
        #     team_2_ally_layers.append(ally_layer(keras.layers.concatenate([combatant_node, ally_node])))
        for foe_node in combatant_nodes[:4]:
            team_2_foe_layers.append(foe_layer(keras.layers.concatenate([combatant_node, foe_node])))

    # ally_combined = dense(layer_size // 6)
    foe_combined = dense(layer_size // 100)

    # team_1_ally_combined = ally_combined(keras.layers.concatenate(team_1_ally_layers))
    team_1_foe_combined = foe_combined(keras.layers.concatenate(team_1_foe_layers))
    # team_2_ally_combined = ally_combined(keras.layers.concatenate(team_2_ally_layers))
    team_2_foe_combined = foe_combined(keras.layers.concatenate(team_2_foe_layers))

    # team_combined = dense(layer_size // 4)
    # team_1_combined = team_combined(keras.layers.concatenate([team_1_ally_combined, team_1_foe_combined]))
    # team_2_combined = team_combined(keras.layers.concatenate([team_2_ally_combined, team_2_foe_combined]))

    concat_all = keras.layers.concatenate([team_1_foe_combined, team_2_foe_combined])
    # concat_all = keras.layers.concatenate([team_1_combined, team_2_combined])
    combined = dense(layer_size // 100)(concat_all)
    predictions = keras.layers.Dense(2, activation='softmax')(combined)

    model = keras.Model(inputs=inputs, outputs=predictions)
    LOG.info(f'Number of parameters: {model.count_params()}')

    model.compile(
        optimizer='nadam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=10, monitor='val_loss', restore_best_weights=True)

    return early_stopping_cb, model


def mc_predict(model, X, samples=100):
    y_probas = np.stack([model.predict(X)
                         for _sample in range(samples)])
    return y_probas.mean(axis=0)


def score_model(model, tag, X, y):
    # predictions = model.predict(X)
    LOG.info(f'Running mc_predict')
    predictions = mc_predict(model, X, samples=20)
    pred_y = np.argmax(predictions, axis=1)
    LOG.info(f'{tag:>8} accuracy   {accuracy_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} precision  {precision_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} recall     {recall_score(y, pred_y):.1%}')
    y_scores = predictions[:, 1]
    LOG.info(f'{tag:>8} roc auc    {roc_auc_score(y, y_scores):.1%}')
    return y_scores


def read_model():
    from tensorflow.keras.models import load_model
    LOG.info(f'Reading model from {config.MODEL_PATH}')
    custom_objects = {MCDropout.__name__: MCDropout}
    return load_model(config.MODEL_PATH, custom_objects=custom_objects, compile=True)


def read_column_transformer():
    LOG.info(f'Reading column transformer from {config.COLUMN_XFORM_PATH}')
    with config.COLUMN_XFORM_PATH.open('rb') as f:
        return pickle.load(f)


def read_feature_scalers():
    LOG.info(f'Reading feature scaler from {config.FEATURE_SCALER_PATH}')
    with config.FEATURE_SCALER_PATH.open('rb') as f:
        return pickle.load(f)


if __name__ == '__main__':
    main()
