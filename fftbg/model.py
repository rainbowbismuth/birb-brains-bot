import logging

import matplotlib.pyplot as plt
from sklearn.compose import ColumnTransformer
from sklearn.metrics import precision_score, recall_score
from sklearn.metrics import roc_curve, roc_auc_score
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, MaxAbsScaler
from tensorflow import keras

import config
import data
import tournament

LOG = logging.getLogger(__name__)


def main():
    LOG.info('Going to compute tournament model')
    df = data.read_matches()

    num_columns = ['Brave', 'Faith']
    cat_columns = ['Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill',
                   'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']

    num_columns = [f'{i + 1}/{c}' for c in num_columns for i in range(8)]
    cat_columns = [f'{i + 1}/{c}' for c in cat_columns for i in range(8)] + ['1/Map']
    skill_columns = [c for c in df.keys() if tournament.SKILL_TAG in c]

    all_columns = num_columns + cat_columns + skill_columns
    dfs = df[all_columns]

    pipeline = ColumnTransformer([
        ('num', MaxAbsScaler(), num_columns),
        ('cat', OneHotEncoder(), cat_columns),
        ('none', 'passthrough', skill_columns),
    ])

    LOG.info('Pre-processing data')
    prepared = pipeline.fit_transform(dfs).astype('float32')

    train_X, test_X, train_y, test_y = train_test_split(prepared, df['1/LeftWins'].to_numpy(), test_size=0.2)
    LOG.info(f'Training data shapes X:{str(train_X.shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes  X:{str(test_X.shape):>14} y:{str(test_y.shape):>9}')

    N = 1000

    model = keras.Sequential(
        [
            keras.layers.Dropout(0.75),
            keras.layers.Dense(N, kernel_initializer='he_normal', use_bias=False),
            keras.layers.BatchNormalization(),
            keras.layers.LeakyReLU(alpha=0.01),
            keras.layers.Dropout(0.75),
            keras.layers.Dense(N, kernel_initializer='he_normal', use_bias=False),
            keras.layers.BatchNormalization(),
            keras.layers.LeakyReLU(alpha=0.01),
            keras.layers.Dropout(0.75),
            keras.layers.Dense(N, kernel_initializer='he_normal', use_bias=False),
            keras.layers.BatchNormalization(),
            keras.layers.LeakyReLU(alpha=0.01),
            keras.layers.Dropout(0.75),
            keras.layers.Dense(2, activation='softmax'),
        ]
    )

    model.compile(optimizer='adam', loss='sparse_categorical_crossentropy', metrics=['accuracy'])
    early_stopping_cb = keras.callbacks.EarlyStopping(patience=10, monitor='val_loss', restore_best_weights=True)
    model.fit(train_X, train_y, epochs=100, verbose=1, validation_split=0.1, callbacks=[early_stopping_cb])

    if config.SAVE_MODEL:
        LOG.info(f'saving model at {config.MODEL_PATH}')
        model.save(config.MODEL_PATH)

    train_pred_y = model.predict_classes(train_X)
    LOG.info(f'training precision  {precision_score(train_y, train_pred_y):.1%}')
    LOG.info(f'training recall     {recall_score(train_y, train_pred_y):.1%}')
    train_y_scores = model.predict(train_X)[:, 1]
    LOG.info(f'training roc auc    {roc_auc_score(train_y, train_y_scores):.1%}')

    test_pred_y = model.predict_classes(test_X)
    LOG.info(f'test precision      {precision_score(test_y, test_pred_y):.1%}')
    LOG.info(f'test recall         {recall_score(test_y, test_pred_y):.1%}')
    test_y_scores = model.predict(test_X)[:, 1]
    LOG.info(f'test roc auc        {roc_auc_score(test_y, test_y_scores):.1%}')

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


if __name__ == '__main__':
    main()
