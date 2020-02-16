from sklearn.base import BaseEstimator


class MyPassthrough(BaseEstimator):
    def __init__(self, column_names=None):
        if column_names is None:
            column_names = []
        self.column_names = column_names

    def fit(self, X, *_args, **_kwargs):
        return X

    def fit_transform(self, X, *_args, **_kwargs):
        return X

    def transform(self, X, *_args, **_kwargs):
        return X

    def get_feature_names(self):
        return self.column_names
