import logging
import pickle
import textwrap

import matplotlib.pyplot as plt
import numpy as np
from sklearn.compose import ColumnTransformer
from sklearn.metrics import roc_curve, roc_auc_score, precision_score, recall_score, accuracy_score, log_loss
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, MaxAbsScaler
from tensorflow import keras

import fftbg.ability
import fftbg.combatant as combatant
import fftbg.config as config
import fftbg.data as data
import fftbg.server
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
CAT_COLUMNS = ['Gender', 'SupportSkill', 'MoveSkill']


def get_status_elemental_columns(df):
    return [c for c in df.keys() if fftbg.combatant.STATUS_ELEMENTAL_TAG in c]


def get_skill_columns(df):
    return [c for c in df.keys() if fftbg.ability.SKILL_TAG in c]


def get_all_columns(df):
    return NUM_COLUMNS + CAT_COLUMNS + get_status_elemental_columns(df) + get_skill_columns(df)


def main():
    fftbg.server.configure_logging('MODEL_LOG_LEVEL')
    LOG.info('Going to compute tournament model')
    df = data.read_combatants()

    skill_columns = get_skill_columns(df)
    all_columns = get_all_columns(df)
    LOG.info('\n'.join(textwrap.wrap(f"All columns: {', '.join(sorted(all_columns))}", 120)))
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

    # tuner = Hyperband(
    #     lambda hp: model_three_hp(hp, combatant_size),
    #     objective='val_loss',
    #     max_epochs=50,
    #     directory='hyperband',
    #     project_name='three-20200229')

    early_stopping_cb, model = model_three(combatant_size)

    # tuner.search(train_X, train_y, epochs=100, verbose=1, validation_data=(valid_X, valid_y))
    model.fit(train_X,
              train_y,
              epochs=3000,
              verbose=1,
              batch_size=1024,
              validation_data=(valid_X, valid_y),
              callbacks=[early_stopping_cb])
    LOG.info('Done training model')

    # model = tuner.get_best_models(num_models=1)[0]

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


def model_three_hp(hp, combatant_size):
    activation = hp.Choice('activation',
                           values=['elu', 'relu'])
    learning_rate = hp.Choice('learning_rate',
                              values=[1e-2, 1e-3, 1e-4])
    extra_size = hp.Choice('extra_size', values=[10, 20, 30, 40, 50, 60, 70, 80, 90, 100])
    extra_layers = hp.Choice('extra_layers', values=[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
    momentum = hp.Choice('momentum', values=[0.9, 0.99, 0.999])
    l1_reg = hp.Float('l1_reg',
                      min_value=0.0005,
                      max_value=0.02)
    l2_reg = hp.Float('l2_reg',
                      min_value=0.0005,
                      max_value=0.01)

    return model_three(combatant_size,
                       extra_size=extra_size,
                       extra_layers=extra_layers,
                       momentum=momentum,
                       activation=activation,
                       l1_rate=l1_reg,
                       l2_rate=l2_reg,
                       learning_rate=learning_rate)[1]


def model_three(combatant_size,
                extra_size=100,
                extra_layers=2,
                momentum=0.99,
                activation='elu',
                l1_rate=0.005,
                l2_rate=0.005,
                learning_rate=0.001):
    def make_dense(output_size):
        dense = keras.layers.Dense(
            output_size,
            kernel_initializer='he_normal',
            kernel_regularizer=keras.regularizers.l1_l2(l1_rate, l2_rate),
            # activation=activation,
            use_bias=False)
        batch = keras.layers.BatchNormalization()
        act = keras.layers.Activation(activation=activation)
        # return dense
        return lambda x: act(batch(dense(x)))

    inputs = [keras.layers.Input(shape=(combatant_size,)) for _ in range(8)]
    first_layer = make_dense(combatant_size // 5)
    first_nodes = [first_layer(input_node) for input_node in inputs]

    nodes = [first_nodes]
    for _ in range(extra_layers):
        new_layer = make_dense(combatant_size // 10)
        nodes.append([new_layer(node) for node in nodes[-1]])
        # nodes.append([new_layer(keras.layers.concatenate(list(a))) for a in zip(inputs, nodes[-1])])

    final_layer = make_dense(combatant_size // 15)
    final_node = final_layer(keras.layers.concatenate(nodes[-1]))

    predictions = keras.layers.Dense(2, activation='softmax')(final_node)

    model = keras.Model(inputs=inputs, outputs=predictions)
    LOG.info(f'Number of parameters: {model.count_params()}')
    model.compile(
        optimizer=keras.optimizers.Nadam(learning_rate=learning_rate),
        loss='sparse_categorical_crossentropy',
        metrics=['accuracy'],
    )

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=100, monitor='val_loss', restore_best_weights=True)

    return early_stopping_cb, model


# TODO: just hacking this to samples=1 since I'm not using MC/Dropout
def mc_predict(model, X, samples=1):
    y_probas = np.stack([model.predict(X)
                         for _sample in range(samples)])
    return y_probas.mean(axis=0)


def score_model(model, tag, X, y):
    # predictions = model.predict(X)
    LOG.info(f'Running mc_predict')
    predictions = mc_predict(model, X, samples=1)
    pred_y = np.argmax(predictions, axis=1)
    LOG.info(f'{tag:>8} accuracy   {accuracy_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} precision  {precision_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} recall     {recall_score(y, pred_y):.1%}')
    LOG.info(f'{tag:>8} log loss   {log_loss(y, predictions):.4}')
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
