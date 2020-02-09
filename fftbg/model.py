from sklearn.compose import ColumnTransformer
from sklearn.model_selection import train_test_split
from sklearn.preprocessing import OneHotEncoder, StandardScaler

import data


def main():
    df = data.read_matches()

    num_columns = ['Brave', 'Faith']
    cat_columns = ['Class', 'Gender', 'Mainhand']

    num_columns = [f'{c}/{i}' for c in num_columns for i in range(8)]
    cat_columns = [f'{c}/{i}' for c in cat_columns for i in range(8)] + ['Map/1']

    all_columns = num_columns + cat_columns
    dfs = df[all_columns]
    print(dfs.keys())

    pipeline = ColumnTransformer([
        ("num", StandardScaler(), num_columns),
        ("cat", OneHotEncoder(), cat_columns)
    ])

    prepared = pipeline.fit_transform(dfs)
    train_X, train_y, test_X, test_y = train_test_split(prepared, df['LeftWins/1'], test_size=0.2)


if __name__ == '__main__':
    main()
