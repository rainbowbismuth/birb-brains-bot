import logging

import matplotlib.pyplot as plt
import numpy as np
from sklearn.compose import ColumnTransformer
from sklearn.metrics import precision_score, recall_score, accuracy_score
from sklearn.metrics import roc_curve, roc_auc_score
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, MaxAbsScaler
from tensorflow import keras

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


def main():
    LOG.info('Going to compute tournament model')
    df = data.read_units()

    num_columns = ['Brave', 'Faith']
    cat_columns = ['Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill',
                   'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory', 'Map']
    skill_columns = [c for c in df.keys() if tournament.SKILL_TAG in c]
    all_columns = num_columns + cat_columns + skill_columns
    dfs = df[all_columns]
    unit_dfs = [df[df['UIDX'] == i][all_columns] for i in range(8)]

    pipeline = ColumnTransformer([
        ('num', MaxAbsScaler(), num_columns),
        ('cat', OneHotEncoder(), cat_columns),
        ('none', 'passthrough', skill_columns),
    ])

    LOG.info('Pre-processing data')
    pipeline.fit(dfs)
    unit_dfs = [pipeline.transform(unit_df).astype('float32') for unit_df in unit_dfs]
    winner = df[df['UIDX'] == 0]['LeftWins']

    train_X, test_X, train_y, test_y = split(unit_dfs, winner, size=0.3)
    test_X, valid_X, test_y, valid_y = split(test_X, test_y, size=0.2)

    # Augment tests:
    train_X2 = train_X[4:] + train_X[:4]
    train_y2 = ~train_y

    train_X = [np.append(train_X[i], train_X2[i], axis=0) for i in range(8)]
    train_y = np.append(train_y, train_y2)

    LOG.info(f'Training data shapes    X:{str(train_X[0].shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes     X:{str(test_X[0].shape):>14} y:{str(test_y.shape):>9}')
    LOG.info(f'Validation data shapes  X:{str(valid_X[0].shape):>14} y:{str(valid_y.shape):>9}')

    UNIT_SIZE = train_X[0].shape[1]
    N = UNIT_SIZE / 2

    def dense(n):
        layer = keras.layers.Dense(
            n,
            kernel_initializer='he_normal',
            activation='elu',
            kernel_regularizer=keras.regularizers.l2(0.01))
        dropout = keras.layers.Dropout(0.25)
        return lambda x: dropout(layer(x))

    unit_inputs = [keras.layers.Input(shape=(UNIT_SIZE,)) for _ in range(8)]
    unit_layer = dense(N)
    unit_nodes = [unit_layer(unit_input) for unit_input in unit_inputs]

    ally_layer = dense(N / 20)
    foe_layer = dense(N / 5)
    pair_layers = []

    for unit_node in unit_nodes[:4]:
        for ally_node in unit_nodes[:4]:
            if unit_node is ally_node:
                continue
            pair_layers.append(ally_layer(keras.layers.concatenate([unit_node, ally_node])))
        for foe_node in unit_nodes[4:]:
            pair_layers.append(foe_layer(keras.layers.concatenate([unit_node, foe_node])))

    for unit_node in unit_nodes[4:]:
        for ally_node in unit_nodes[4:]:
            if unit_node is ally_node:
                continue
            pair_layers.append(ally_layer(keras.layers.concatenate([unit_node, ally_node])))
        for foe_node in unit_nodes[:4]:
            pair_layers.append(foe_layer(keras.layers.concatenate([unit_node, foe_node])))

    concat_all = keras.layers.concatenate(pair_layers)
    combined = dense(N)(concat_all)
    predictions = keras.layers.Dense(2, activation='softmax')(combined)

    model = keras.Model(inputs=unit_inputs, outputs=predictions)
    model.compile(optimizer='adam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])
    early_stopping_cb = keras.callbacks.EarlyStopping(patience=10, monitor='val_loss', restore_best_weights=True)
    model.fit(train_X,
              train_y,
              epochs=100,
              verbose=1,
              validation_data=(valid_X, valid_y),
              callbacks=[early_stopping_cb])

    if config.SAVE_MODEL:
        LOG.info(f'saving model at {config.MODEL_PATH}')
        model.save(config.MODEL_PATH)

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


def score_model(model, tag, X, y):
    predictions = model.predict(X)
    pred_y = np.argmax(predictions, axis=1)
    LOG.info(f'{tag:>8} accuracy   {accuracy_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} precision  {precision_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} recall     {recall_score(y, pred_y):.1%}')
    y_scores = predictions[:, 1]
    LOG.info(f'training roc auc    {roc_auc_score(y, y_scores):.1%}')
    return y_scores


if __name__ == '__main__':
    main()
