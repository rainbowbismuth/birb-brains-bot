import logging
import pickle
from typing import List

import matplotlib.pyplot as plt
import numpy as np
import tensorflow
from kerastuner import Hyperband
from sklearn.compose import ColumnTransformer
from sklearn.metrics import roc_curve, roc_auc_score, precision_score, recall_score, accuracy_score, log_loss
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, MaxAbsScaler
from tensorflow import keras
from tensorflow import linalg

import fftbg.ability
import fftbg.combatant as combatant
import fftbg.config as config
import fftbg.data as data
import fftbg.tournament as tournament
from fftbg.passthrough import MyPassthrough

LOG = logging.getLogger(__name__)


def split(xs, y, size):
    splits = train_test_split(*xs, y, test_size=size)
    train_y = splits[-2]
    test_y = splits[-1]
    train_X = [splits[i * 2] for i in range(8)]
    test_X = [splits[i * 2 + 1] for i in range(8)]
    return train_X, test_X, train_y, test_y


NUM_COLUMNS = combatant.NUMERIC + tournament.NUMERIC
CAT_COLUMNS = ['Gender', 'Class', 'SupportSkill', 'MoveSkill']


def get_skill_columns(df):
    return [c for c in df.keys() if fftbg.ability.SKILL_TAG in c]


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
        ('c',
         OneHotEncoder(handle_unknown="ignore"),
         CAT_COLUMNS),
        ('p',
         MyPassthrough(NUM_COLUMNS + skill_columns),
         NUM_COLUMNS + skill_columns),
    ])

    LOG.info('Pre-processing data')
    pipeline.fit(dfs)
    combatant_dfs = [pipeline.transform(combatant_df).astype('float32')
                     for combatant_df in combatant_dfs]
    winner = df[df['UIDX'] == 0]['LeftWins']

    train_X, test_X, train_y, test_y = split(combatant_dfs, winner, size=0.3)
    test_X, valid_X, test_y, valid_y = split(test_X, test_y, size=0.2)

    scalers = [MaxAbsScaler() for _ in range(len(train_X))]
    train_X = [scaler.fit_transform(train_xi) for (scaler, train_xi) in zip(scalers, train_X)]
    test_X = [scaler.transform(test_xi) for (scaler, test_xi) in zip(scalers, test_X)]
    valid_X = [scaler.transform(valid_xi) for (scaler, valid_xi) in zip(scalers, valid_X)]

    if config.SAVE_MODEL:
        LOG.info(f'Saving column transformation pipeline at {config.COLUMN_XFORM_PATH}')
        with config.COLUMN_XFORM_PATH.open(mode='wb') as f:
            pickle.dump(pipeline, f)
        LOG.info(f'Saving feature scalers at {config.FEATURE_SCALER_PATH}')
        with config.FEATURE_SCALER_PATH.open(mode='wb') as f:
            pickle.dump(scalers, f)

    # combined_training = np.concatenate(train_X)
    # combined_training_y = np.concatenate([train_y for _y in range(8)])
    # sel1 = VarianceThreshold(0.98 * (1-0.98))
    # sel2 = SelectKBest(chi2, k=100)
    # combined_training = sel1.fit_transform(combined_training)
    # sel2.fit(combined_training, combined_training_y)
    #
    # sel1_support = sel1.get_support()
    # sel2_support = sel2.get_support()
    # word_vec = pipeline.get_feature_names()
    # word_vec = ma.masked_array(word_vec, ~sel1_support).compressed()
    # word_vec = ma.masked_array(word_vec, ~sel2_support).compressed()
    # important_features = [word.replace('__', '') for word in sorted(word_vec)]
    # print(f"determined the following {len(important_features)} important features:")
    # print(textwrap.fill(", ".join(sorted(important_features)), width=120))
    # return

    # def rotate_right(xs, amount: int = 1):
    #     return xs[-amount:] + xs[:-amount]

    # Augment tests:
    # train_X2 = rotate_right(train_X[:4]) + rotate_right(train_X[4:])
    # train_y2 = train_y  # ~train_y
    #
    # train_X = [np.append(train_X[i], train_X2[i], axis=0) for i in range(8)]
    # train_y = np.append(train_y, train_y2)

    LOG.info(f'Training data shapes    X:{str(train_X[0].shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes     X:{str(test_X[0].shape):>14} y:{str(test_y.shape):>9}')
    LOG.info(f'Validation data shapes  X:{str(valid_X[0].shape):>14} y:{str(valid_y.shape):>9}')

    combatant_size = train_X[0].shape[1]

    tuner = Hyperband(
        lambda hp: model_residual_hp(hp, combatant_size),
        objective='val_loss',
        max_epochs=30,
        directory='hyperband',
        project_name='residual-20200217')

    # early_stopping_cb, model = model_residual(combatant_size,
    #                                           activation='relu',
    #                                           kernel_size=0.05,
    #                                           learning_rate=1e-3,
    #                                           drop_out_input=0.35,
    #                                           drop_out_res=0.35,
    #                                           drop_out_final=0.5,
    #                                           l2_reg=0.005)
    # early_stopping_cb, model = model_huge_multiply(combatant_size)
    tuner.search(train_X, train_y, epochs=100, verbose=1, validation_data=(valid_X, valid_y))
    # model.fit(train_X,
    #           train_y,
    #           epochs=100,
    #           verbose=1,
    #           validation_data=(valid_X, valid_y),
    #           callbacks=[early_stopping_cb])
    LOG.info('Done training model')

    model = tuner.get_best_models(num_models=2)[0]

    if config.SAVE_MODEL:
        LOG.info(f'Saving model at {config.MODEL_PATH}')
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


class MCDropout(keras.layers.Dropout):
    def call(self, inputs, _training=True):
        return super().call(inputs, training=True)


def model_huge_multiply(combatant_size):
    combatant_layer = keras.layers.Dense(
        combatant_size // 2,
        kernel_initializer='he_normal',
        kernel_regularizer=keras.regularizers.l2(0.005),
        activation='relu',
        use_bias=True)

    inputs = [keras.layers.Input(shape=(combatant_size,)) for _ in range(8)]
    combatant_nodes = [combatant_layer(input_node) for input_node in inputs]

    pair_multiply_layer = keras.layers.Dense(
        2,
        kernel_initializer='he_normal',
        kernel_regularizer=keras.regularizers.l2(0.005),
        activation='elu',
        use_bias=True)

    def multiply(tensors: List[tensorflow.Tensor]):
        size = tensors[0].shape[1]

        def fn(xs):
            return [tensorflow.reshape(
                linalg.matmul(
                    tensorflow.reshape(xs[0], (-1, 1)),
                    tensorflow.reshape(xs[1], (1, -1)),
                    a_is_sparse=True,
                    b_is_sparse=True,
                ),
                (size ** 2,)), 0]

        return tensorflow.map_fn(fn,
                                 elems=tensors)[0]

    def pair_multiply(a, b):
        return keras.layers.Lambda(multiply)([a, b])

    foe_nodes = []
    for p1 in combatant_nodes[:4]:
        for p2 in combatant_nodes[4:]:
            foe_nodes.append(pair_multiply_layer(pair_multiply(p1, p2)))

    predictions = keras.layers.Dense(2, activation='softmax')(
        keras.layers.concatenate(foe_nodes))

    model = keras.Model(inputs=inputs, outputs=predictions)
    LOG.info(f'Number of parameters: {model.count_params()}')
    model.compile(
        optimizer=keras.optimizers.Nadam(learning_rate=1e-3),
        loss='sparse_categorical_crossentropy',
        metrics=['accuracy'])

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=10, monitor='val_loss', restore_best_weights=True)

    return early_stopping_cb, model


def model_residual_hp(hp, combatant_size):
    activation = hp.Choice('activation',
                           values=['elu', 'relu', 'tahn', 'sigmoid'])
    kernel_size = hp.Float('kernel_size',
                           min_value=0.05,
                           max_value=1.0)
    learning_rate = hp.Choice('learning_rate',
                              values=[1e-2, 1e-3, 1e-4])
    drop_out_input = hp.Float('drop_out_input',
                              min_value=0.0,
                              max_value=0.5)
    drop_out_res = hp.Float('drop_out_res',
                            min_value=0.0,
                            max_value=0.5)
    drop_out_final = hp.Float('drop_out_final',
                              min_value=0.0,
                              max_value=0.5)
    l2_reg = hp.Float('l2_reg',
                      min_value=0.0,
                      max_value=0.02)
    return model_residual(combatant_size, activation,
                          kernel_size, learning_rate, drop_out_input,
                          drop_out_res, drop_out_final, l2_reg)[1]


def model_residual(combatant_size, activation,
                   kernel_size, learning_rate, drop_out_input,
                   drop_out_res, drop_out_final, l2_reg):
    def res_block(n=combatant_size):
        k_size = int(kernel_size * n)
        layer_1 = keras.layers.Dense(
            k_size,
            kernel_initializer='he_normal',
            kernel_regularizer=keras.regularizers.l2(l2_reg),
            activation=activation,
            use_bias=True)

        layer_2 = keras.layers.Dense(
            n,
            kernel_initializer='he_normal',
            kernel_regularizer=keras.regularizers.l2(l2_reg),
            activation=activation,
            use_bias=True)
        do = MCDropout(drop_out_res)

        def combine(x):
            fg = layer_2(layer_1(x))
            add = keras.layers.add([fg, x])
            return do(add)

        return combine

    inputs = [keras.layers.Input(shape=(combatant_size,)) for _ in range(8)]
    combatant_layer = res_block()
    combatant_nodes = [MCDropout(drop_out_input)(combatant_layer(node))
                       for node in inputs]

    f1_layer = res_block(combatant_size * 2)
    foe_nodes = []
    for p1 in combatant_nodes[:4]:
        for p2 in combatant_nodes[4:]:
            sub = keras.layers.concatenate([p1, p2])
            node = f1_layer(sub)
            foe_nodes.append(node)

    team_stack1 = keras.layers.maximum(combatant_nodes[:4])
    team_stack2 = keras.layers.maximum(combatant_nodes[4:])
    team_diff = keras.layers.subtract([team_stack1, team_stack2])
    team_computed = res_block()(team_diff)

    combined = MCDropout(drop_out_final)(keras.layers.average(foe_nodes))
    predictions = keras.layers.Dense(2, activation='softmax')(
        keras.layers.concatenate([combined, team_computed]))

    model = keras.Model(inputs=inputs, outputs=predictions)
    LOG.info(f'Number of parameters: {model.count_params()}')

    model.compile(
        optimizer=keras.optimizers.Nadam(learning_rate=learning_rate),
        loss='sparse_categorical_crossentropy',
        metrics=['accuracy'])

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=10, monitor='val_loss', restore_best_weights=True)

    return early_stopping_cb, model


def dense_bias(n):
    d = keras.layers.Dense(
        n,
        kernel_initializer='he_normal',
        kernel_regularizer=keras.regularizers.l2(0.01),
        activation='elu',
        use_bias=True)
    return d


def dense_norm(n):
    d = keras.layers.Dense(
        n,
        kernel_initializer='he_normal',
        kernel_regularizer=keras.regularizers.l2(0.01),
        use_bias=False)
    bn = keras.layers.BatchNormalization()
    a = keras.layers.Activation('elu')
    return lambda x: a(bn(d(x)))


def dense(n, o=None, p=None):
    if o is None:
        o = n
    if p is None:
        p = o
    d1 = dense_bias(n)
    d2 = dense_bias(o)
    d3 = dense_norm(p)
    mc = MCDropout(0.50)
    return lambda x: mc(d3(d2(d1(x))))


def model_classic(combatant_size, layer_size):
    inputs = [keras.layers.Input(shape=(combatant_size,)) for _ in range(8)]
    combatant_layer = dense(layer_size, layer_size // 5, layer_size // 10)
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
            team_1_foe_layers.append(foe_layer(keras.layers.subtract([combatant_node, foe_node])))

    for combatant_node in combatant_nodes[4:]:
        # for ally_node in combatant_nodes[4:]:
        #     if combatant_node is ally_node:
        #         continue
        #     team_2_ally_layers.append(ally_layer(keras.layers.concatenate([combatant_node, ally_node])))
        for foe_node in combatant_nodes[:4]:
            team_2_foe_layers.append(foe_layer(keras.layers.subtract([combatant_node, foe_node])))

    # ally_combined = dense(layer_size // 6)
    foe_combined = dense(layer_size // 100)

    # team_1_ally_combined = ally_combined(keras.layers.concatenate(team_1_ally_layers))
    # team_1_foe_combined = foe_combined(keras.layers.concatenate(team_1_foe_layers))
    team_1_foe_combined = foe_combined(keras.layers.average(team_1_foe_layers))
    # team_2_ally_combined = ally_combined(keras.layers.concatenate(team_2_ally_layers))
    # team_2_foe_combined = foe_combined(keras.layers.concatenate(team_2_foe_layers))
    team_2_foe_combined = foe_combined(keras.layers.average(team_2_foe_layers))

    # team_combined = dense(layer_size // 4)
    # team_1_combined = team_combined(keras.layers.concatenate([team_1_ally_combined, team_1_foe_combined]))
    # team_2_combined = team_combined(keras.layers.concatenate([team_2_ally_combined, team_2_foe_combined]))

    # concat_all = keras.layers.concatenate([team_1_foe_combined, team_2_foe_combined])
    concat_all = keras.layers.subtract([team_1_foe_combined, team_2_foe_combined])
    # concat_all = keras.layers.concatenate([team_1_combined, team_2_combined])
    combined = dense(layer_size // 100)(concat_all)
    predictions = keras.layers.Dense(2, activation='softmax')(combined)

    model = keras.Model(inputs=inputs, outputs=predictions)
    LOG.info(f'Number of parameters: {model.count_params()}')

    model.compile(
        optimizer='nadam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=20, monitor='val_loss', restore_best_weights=True)

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
    LOG.info(f'{tag:>8} log loss   {log_loss(y, pred_y):.4}')
    y_scores = predictions[:, 1]
    LOG.info(f'{tag:>8} roc auc    {roc_auc_score(y, y_scores):.1%}')
    return y_scores


def read_model():
    from tensorflow.keras.models import load_model
    LOG.info(f'Reading model from {config.MODEL_PATH}')
    custom_objects = {
        MCDropout.__name__: MCDropout,
    }
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
