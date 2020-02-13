import data
import download
import tournament


def update_data():
    download.tournament_sync()
    tournaments = tournament.parse_tournaments()
    df = tournament.tournaments_to_units(tournaments)
    data.write_units(df)


if __name__ == '__main__':
    update_data()
