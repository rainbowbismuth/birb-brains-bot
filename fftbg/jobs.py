import tournament
import download
import data


def update_unit_data():
    download.tournament_sync()
    df = tournament.parse_all_units()
    data.write_units(df)


if __name__ == '__main__':
    update_unit_data()
