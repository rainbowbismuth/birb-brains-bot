from sklearn.compose import ColumnTransformer
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder

import fftbg.server
import fftbg.bird.memory
import logging
import pandas
from datetime import datetime
from tensorflow import keras

from model import split
from passthrough import MyPassthrough

LOG = logging.getLogger(__name__)

CAT_COLUMNS = ['left_team', 'right_team', 'hour']
NUM_COLUMNS = ['time', 'left_total_on_bet', 'right_total_on_bet', 'left_prediction', 'right_prediction']
OUTPUT_COLUMNS = ['left_total_final', 'right_total_final']
ALL_COLUMNS = CAT_COLUMNS + NUM_COLUMNS


def main():
    fftbg.server.configure_logging('MODEL_LOG_LEVEL')
    LOG.info('Going to compute betting model')
    memory = fftbg.bird.memory.Memory()
    balance_log = memory.get_balance_log(limit=1_000_000)

    data = []

    for log in balance_log:
        if log.left_total_final == 0 or log.right_total_final == 0 \
                or log.left_total_on_bet == 0 or log.right_total_on_bet == 0:
            continue

        entry = {}
        time = datetime.fromisoformat(log.time)
        entry['time'] = (time.hour * 60 + time.minute) / (60 * 24.0)
        entry['hour'] = str(time.hour)
        entry['left_team'] = log.left_team
        entry['right_team'] = log.right_team
        entry['left_prediction'] = log.left_prediction
        entry['right_prediction'] = log.right_prediction
        entry['left_total_final'] = log.left_total_final / (log.left_total_final + log.right_total_final)
        entry['right_total_final'] = log.right_total_final / (log.left_total_final + log.right_total_final)
        entry['left_total_on_bet'] = log.left_total_on_bet / (log.left_total_on_bet + log.right_total_on_bet)
        entry['right_total_on_bet'] = log.right_total_on_bet / (log.left_total_on_bet + log.right_total_on_bet)
        data.append(entry)

    df = pandas.DataFrame(data)

    pipeline = ColumnTransformer([
        ('c',
         OneHotEncoder(handle_unknown="ignore"),
         CAT_COLUMNS),
        ('p',
         MyPassthrough(NUM_COLUMNS),
         NUM_COLUMNS),
    ])

    dfs = pandas.DataFrame(pipeline.fit_transform(df[ALL_COLUMNS]).toarray())
    output = df[OUTPUT_COLUMNS]

    train_X, test_X, train_y, test_y = train_test_split(dfs, output, test_size=0.3)
    test_X, valid_X, test_y, valid_y = train_test_split(test_X, test_y, test_size=0.2)

    LOG.info(f'Training data shapes    X:{str(train_X.shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes     X:{str(test_X.shape):>14} y:{str(test_y.shape):>9}')
    LOG.info(f'Validation data shapes  X:{str(valid_X.shape):>14} y:{str(valid_y.shape):>9}')

    row_size = train_X.shape[1]
    early_stopping_cb, model = model_one(row_size)
    model.fit(train_X,
              train_y,
              epochs=2000,
              verbose=1,
              batch_size=256,
              validation_data=(valid_X, valid_y),
              callbacks=[early_stopping_cb])
    LOG.info('Done training model')

    predicted = model.predict(test_X)
    for i in range(10):
        print(test_X.iloc[i])
        print(predicted[i, :])
        print(test_y.iloc[i])


def model_one(row_size,
              activation='elu',
              l1_rate=0.005,
              l2_rate=0.005,
              learning_rate=0.001):
    def make_dense(output_size):
        return keras.layers.Dense(
            output_size,
            kernel_initializer='he_normal',
            kernel_regularizer=keras.regularizers.l1_l2(l1_rate, l2_rate),
            activation=activation,
            use_bias=False)

    input_layer = keras.layers.Input(shape=(row_size,))
    layer_one = make_dense(row_size)(input_layer)
    layer_two = make_dense(row_size)(layer_one)
    # layer_three = make_dense(row_size)(layer_two)
    output = keras.layers.Dense(2, activation='elu')(layer_two)

    model = keras.Model(inputs=input_layer, outputs=output)
    LOG.info(f'Number of parameters: {model.count_params()}')
    model.compile(
        optimizer=keras.optimizers.Nadam(learning_rate=learning_rate),
        loss='mse',
        metrics=['mae', 'mse'],
    )

    early_stopping_cb = keras.callbacks.EarlyStopping(
        patience=20, monitor='val_loss', restore_best_weights=True)

    return early_stopping_cb, model



if __name__ == '__main__':
    main()
