'use strict';

const State = {
    balance_log: [],
    placed_bet: null
};

function load_summary_info() {
    m.request({
        method: 'GET',
        url: '/balance-log',
    }).then(function (result) {
        State.balance_log = result;
        State.balance_log.forEach(function (log) {
            log.time = moment(log.time + 'Z');
        });
        State.balance_log.sort((a, b) => b.id - a.id);
    });

    m.request({
        method: 'GET',
        url: '/placed-bet',
    }).then(function (result) {
        State.placed_bet = result;
        State.placed_bet.time = moment(result.time + 'Z');
    });
}

function display_gain_loss(x) {
    if (x === 0) {
        return m('span.text-muted', '0');
    } else if (x > 0) {
        return m('span.text-success', '+' + x.toLocaleString());
    } else {
        return m('span.text-danger', x.toLocaleString());
    }
}

function win_lose(win) {
    return win ? m('small.text-muted', 'W') : m('small.text-muted', 'L');
}

function team_color(team) {
    return m('span.just-right.color-' + team, team);
}

function format_prediction(percent) {
    return (percent * 100).toFixed(1) + '%';
}

const StatSummary = {
    view: function (vnode) {
        if (State.balance_log.length === 0) {
            return '';
        }
        const length = State.balance_log.length;
        const oldest = State.balance_log[length - 1].time;
        const newest = State.balance_log[0].time;
        const duration = moment.duration(newest.diff(oldest));
        const hours_shown = duration.asHours();
        const total_gain = State.balance_log
            .map(log => log.new_balance - log.old_balance)
            .reduce((a, b) => a + b, 0);
        const total_wager = State.balance_log
            .map(log => log.wager)
            .reduce((a, b) => a + b, 0);

        const placed_bet = State.placed_bet;
        let placed_bet_msg = m('div', [
            'Betting is currently open!'
        ]);
        if (placed_bet && newest.isBefore(placed_bet.time)) {
            const win_prediction =
                placed_bet.left_team === placed_bet.bet_on
                    ? placed_bet.left_prediction
                    : placed_bet.right_prediction;

            placed_bet_msg = m('div', [
                'Betting ', placed_bet.wager.toLocaleString(), ' G on ',
                team_color(placed_bet.bet_on),
                m('span.text-muted',
                    ' (Estimated ', format_prediction(win_prediction), ' chance to win.)')
            ]);
        }

        return m('.card.stat-summary', [
            m('.card-body', [
                m('h5.card-title', 'Quick summary'),
                m('.card-text', [
                    m('div', [length, ' matches ',
                        m('span.text-muted', 'over the last '),
                        duration.hours(), ' hours ',
                        m('span.text-muted', 'are shown on this page.')]),
                    m('div', [
                        m('span.text-muted', 'Total Gain/Loss: '),
                        display_gain_loss(total_gain), ' G',
                        ' (', display_gain_loss((total_gain / hours_shown) | 0), ' G/hour.)'
                    ]),
                    m('div', [
                        m('span.text-muted', 'Total Wagers: '),
                        total_wager.toLocaleString(), ' G',
                        ' (', ((total_wager / hours_shown) | 0).toLocaleString(), ' G/hour.)'
                    ]),
                    placed_bet_msg
                ])
            ])
        ]);
    }
};

const BalanceLog = {
    view: function (vnode) {
        return m('table.table', [
            m('thead', [
                m('tr', [
                    m('th', {scope: 'col'}, '#'),
                    m('th', {scope: 'col'}, 'Time'),
                    m('th', {scope: 'col'}, 'Tournament ID'),
                    m('th', {scope: 'col'}, 'Balance'),
                    m('th', {scope: 'col'}, 'Gain/Loss'),
                    m('th', {scope: 'col'}, 'Wager'),
                    m('th', ''),
                    m('th', {scope: 'col'}, 'Player 1'),
                    m('th', {scope: 'col'}, 'Est. Win%'),
                    m('th', {scope: 'col'}, 'Bet Total'),
                    m('th', ''),
                    m('th', {scope: 'col'}, 'Player 2'),
                    m('th', {scope: 'col'}, 'Est. Win%'),
                    m('th', {scope: 'col'}, 'Bet Total'),
                ]),
            ]),
            m('tbody', State.balance_log.map(function (log) {
                return m('tr', [
                    m('th.text-muted', {scope: 'row'}, log.id),
                    m('td', log.time.format('LT')),
                    m('td.text-muted', log.tournament),
                    m('td.jr', log.new_balance.toLocaleString()),
                    m('td.jr', display_gain_loss(log.new_balance - log.old_balance)),
                    m('td.jr', log.wager.toLocaleString()),
                    m('td', win_lose(log.left_wins)),
                    m('td', team_color(log.left_team)),
                    m('td', format_prediction(log.left_prediction)),
                    m('td.jr.text-muted', log.left_total_final.toLocaleString()),
                    m('td', win_lose(!log.left_wins)),
                    m('td', team_color(log.right_team)),
                    m('td', format_prediction(log.right_prediction)),
                    m('td.jr.text-muted', log.right_total_final.toLocaleString()),
                ]);
            }))
        ]);
    }
};

const Root = {
    view: function (vnode) {
        return [
            m('.container', [
                m('.row', [
                    m('.col', [
                        m('nav.navbar.navbar-expand-lg.navbar-dark.bg-dark', [
                            m('a.navbar-brand', {href: '#'}, 'Birb Brains Bot'),
                            m('span.navbar-text', [
                                'A machine learning powered betting bot for ',
                                m('a', {href: 'https://www.twitch.tv/fftbattleground'}, 'FFT Battleground')
                            ])
                        ]),
                        m('br'),
                        m('p', 'This work in progress dashboard is to help me get an idea of how the bot is ' +
                            'doing without having to dig through log files, but feel free to take a look yourself.')
                    ])
                ]),
                m('.row', [
                    m('.col', [
                        m(StatSummary),
                        m(BalanceLog)
                    ])
                ])
            ])
        ];
    }
};

m.mount(document.body, Root);
load_summary_info();

document.addEventListener("visibilitychange", function () {
        if (document.visibilityState === 'visible') {
            load_summary_info();
        }
    }
);