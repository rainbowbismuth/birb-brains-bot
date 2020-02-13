import base_stats
import equipment

PER_COMBATANTS = ['Name', 'Gender', 'Sign', 'Class', 'ActionSkill', 'ReactionSkill', 'SupportSkill', 'MoveSkill',
                  'Mainhand', 'Offhand', 'Head', 'Armor', 'Accessory']
NUMERIC = ['Brave', 'Faith', 'HP', 'HP+Armor', 'MP', 'MP+Armor']
CATEGORICAL = PER_COMBATANTS + ['Color', 'Side', 'Map']
SKILL_TAG = '⭒ '


def combatant_to_dict(combatant):
    skills = {}
    for skill in combatant['ClassSkills']:
        skills[SKILL_TAG + skill] = True
    for skill in combatant['ExtraSkills']:
        skills[SKILL_TAG + skill] = True
    output = {**combatant, **skills}
    del output['ClassSkills']
    del output['ExtraSkills']

    # Compute stats
    stats = base_stats.get_base_stats(combatant['Class'], combatant['Gender'])
    headgear = equipment.get_equipment(combatant['Head'])
    armor = equipment.get_equipment(combatant['Armor'])

    output['HP'] = stats.hp
    output['HP+Armor'] = stats.hp + headgear.hp_bonus + armor.hp_bonus
    output['MP'] = stats.mp
    output['MP+Armor'] = stats.mp + headgear.mp_bonus + armor.mp_bonus

    return output
