import re


def re_compile(regex):
    return re.compile(regex, re.IGNORECASE)


NEW_TOURNAMENT = 'You may now !fight to enter the tournament!'
NEW_TOURNAMENT_SKILL_DROP = re_compile(r'This tournament\'s Skill Drop is: ([\w\d\-+]+)\.')

BET_RE = re_compile(r'!bet (\w+) ([\d%]+)')
BET2_RE = re_compile(r'!bet ([\d%]+) (\w+)')
ALL_IN_RE = re_compile(r'!allin (\w+)')
BOTS_CANNOT_BET_RE = re_compile(r'(\w+), bots cannot bet in the final (\d+) seconds!')
BALANCE_RE = re_compile(r'(\w+), your balance is: ([\d,]+)G')
BALANCE2_RE = re_compile(r'(\w+), your bettable balance is: ([\d,]+)G')
BETTING_OPEN_RE = re_compile(r'Betting is open for (\w+) vs (\w+).')
BETTING_CLOSE_RE = re_compile(r'Betting is closed. Final Bets: (\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) '
                              r'bets for ([\d,]+)G')
ODDS_RE = re_compile(r'(\w+) - (\d+) bets for ([\d,]+)G(?:.*?); (\w+) - (\d+) bets for ([\d,]+)G')
BETTING_CLOSED_RE = re_compile(r'(\w+), betting has closed, sorry!')
TEAM_VICTORY = re_compile(r'The (\w+) team was victorious!')

SKILL_PURCHASE = re_compile(r'([\w\d_]+), you bought the ([\w\d\-+]+) skill')
SKILL_LEARN = re_compile(r'([\w\d_]+),.*?You learned the skill: ([\w\d\-+]+)!')
SKILL_GIFT = re_compile(r'Due to a generous donation from ([\w\d_]+), ([\w\d_]+) has been bestowed the ([\w\d\-+]+) '
                        r'skill')
SKILL_BESTOW_1 = re_compile(r'([\w\d_]+)! You have been bestowed the ([\w\d\-+]+) skill free of charge!')
SKILL_BESTOW_2 = re_compile(r'([\w\d\-+]+) skill free of charge! Additionally, ([\w\d_]+)')
SKILL_RANDOM = re_compile(r'([\w\d_]+), you rolled the dice and bought the ([\w\d\-+]+)')
SKILL_REMEMBERED = re_compile(r'([\w\d_]+), you advanced to Level ([\d,]+)! Your gil floor has increased to ([\d,]+)! You remembered the skill: ([\w\d\-+]+)!')

def parse_comma_int(s):
    return int(s.replace(',', ''))
