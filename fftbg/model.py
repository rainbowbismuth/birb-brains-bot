import logging

import matplotlib.pyplot as plt
from sklearn.compose import ColumnTransformer
from sklearn.linear_model import SGDClassifier
from sklearn.metrics import precision_score, recall_score
from sklearn.metrics import roc_curve, roc_auc_score
from sklearn.model_selection import GridSearchCV, cross_val_predict
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, StandardScaler

import data
import tournament

LOG = logging.getLogger(__name__)


def main():
    LOG.info('Going to compute tournament model')
    df = data.read_matches()

    num_columns = ['Brave', 'Faith']
    cat_columns = ['Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill',
                   'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']

    num_columns = [f'{c}/{i}' for c in num_columns for i in range(8)]
    cat_columns = [f'{c}/{i}' for c in cat_columns for i in range(8)] + ['Map/1']
    skill_columns = [c for c in df.keys() if c.startswith(tournament.SKILL_TAG)]

    all_columns = num_columns + cat_columns + skill_columns
    dfs = df[all_columns]

    pipeline = ColumnTransformer([
        ('num', StandardScaler(), num_columns),
        ('cat', OneHotEncoder(), cat_columns),
        ('none', 'passthrough', skill_columns),
    ])

    LOG.info('Pre-processing data')
    prepared = pipeline.fit_transform(dfs).astype('float32')

    train_X, test_X, train_y, test_y = train_test_split(prepared, df['LeftWins/1'], test_size=0.2)
    LOG.info(f'Training data shapes X:{str(train_X.shape):>14} y:{str(train_y.shape):>9}')
    LOG.info(f'Testing data shapes  X:{str(test_X.shape):>14} y:{str(test_y.shape):>9}')

    param_grid = [
        {'loss': ['hinge', 'log', 'modified_huber', 'squared_hinge', 'perceptron'],
         'penalty': ['l2', 'l1', 'elasticnet']}
    ]
    clf = SGDClassifier()
    grid_search = GridSearchCV(clf, param_grid, cv=5, n_jobs=-1)

    LOG.info(f'Beginning GridSearchCV')
    grid_search.fit(train_X, train_y)

    LOG.info(f'Best parameters found {grid_search.best_params_}')
    best_clf = grid_search.best_estimator_

    train_pred_y = cross_val_predict(best_clf, train_X, train_y, cv=5)
    LOG.info(f'training precision  {precision_score(train_y, train_pred_y):.1%}')
    LOG.info(f'training recall     {recall_score(train_y, train_pred_y):.1%}')

    # test_pred_y = best_clf.predict(test_X)
    # LOG.info(f'test precision      {precision_score(test_y, test_pred_y):.1%}')
    # LOG.info(f'test recall         {recall_score(test_y, test_pred_y):.1%}')

    train_y_scores = cross_val_predict(best_clf, train_X, train_y, cv=5, method='decision_function')
    LOG.info(f'training roc auc    {roc_auc_score(train_y, train_y_scores):.1%}')

    fpr, tpr, thresholds = roc_curve(train_y, train_y_scores)
    plt.plot(fpr, tpr, linewidth=2)
    plt.plot([0, 1], [0, 1], 'k--')
    plt.xlabel('False Positive Rate')
    plt.ylabel('True Positive Rate (Recall)')
    plt.grid()
    plt.show()


if __name__ == '__main__':
    main()
