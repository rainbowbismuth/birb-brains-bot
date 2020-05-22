import re

NEW_TOURNAMENT = 'You may now !fight to enter the tournament!'
NEW_TOURNAMENT_SKILL_DROP = re.compile(r'This tournament\'s Skill Drop is: ([\w\d\-+]+)\.')

BET_RE = re.compile(r'!bet (\w+) ([\d%]+)')
BET2_RE = re.compile(r'!bet ([\d%]+) (\w+)')
ALL_IN_RE = re.compile(r'!allin (\w+)')
BOTS_CANNOT_BET_RE = re.compile(r'(\w+), bots cannot bet in the final (\d+) seconds!')
BALANCE_RE = re.compile(r'(\w+), your balance is: ([\d,]+)G')
BALANCE2_RE = re.compile(r'(\w+), your bettable balance is: ([\d,]+)G')
BETTING_OPEN_RE = re.compile(r'Betting is open for (\w+) vs (\w+).')
BETTING_CLOSE_RE = re.compile(r'Betting is closed: Final Bets: (\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) '
                              r'bets for ([\d,]+)G')
ODDS_RE = re.compile(r'(\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) bets for ([\d,]+)G')
BETTING_CLOSED_RE = re.compile(r'(\w+), betting has closed, sorry!')
TEAM_VICTORY = re.compile(r'The (\w+) team was victorious!')


def parse_comma_int(s):
    return int(s.replace(',', ''))
