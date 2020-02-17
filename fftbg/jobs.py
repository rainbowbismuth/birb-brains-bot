import fftbg.data as data
import fftbg.download as download
import fftbg.tournament as tournament


def update_data():
    download.tournament_sync()
    tournaments = tournament.parse_tournaments()
    df = tournament.tournament_to_combatants(tournaments)
    data.write_combatants(df)


if __name__ == '__main__':
    update_data()
