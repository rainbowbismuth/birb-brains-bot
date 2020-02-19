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

        const log_losses_sum = State.balance_log
            .map(log => {
                if (log.left_wins) {
                    return -Math.log(log.left_prediction);
                } else {
                    return -Math.log(1 - log.left_prediction);
                }
            }).reduce((a, b) => a + b, 0);
        const average_log_loss = log_losses_sum / length;

        const accuracy_sum = State.balance_log
            .map(log => {
                if (log.left_wins && log.left_prediction > 0.5) {
                    return 1;
                } else if (!log.left_wins && log.right_prediction > 0.5) {
                    return 1;
                } else {
                    return 0;
                }
            }).reduce((a, b) => a + b, 0);
        const accuracy_average = accuracy_sum / length;

        const placed_bet = State.placed_bet;
        let placed_bet_msg = m('div', [
            'Betting is currently open!'
        ]);
        if (placed_bet && newest.isBefore(placed_bet.time)) {
            const win_prediction =
                placed_bet.left_team === placed_bet.bet_on
                    ? placed_bet.left_prediction
                    : placed_bet.right_prediction;

            const not_bet_team =
                placed_bet.bet_on === placed_bet.left_team
                    ? placed_bet.right_team
                    : placed_bet.left_team;

            placed_bet_msg = m('div', [
                'Betting ', placed_bet.wager.toLocaleString(), ' G on ',
                team_color(placed_bet.bet_on),
                m('span.text-muted',
                    ' (', format_prediction(win_prediction), ' chance to win versus ',
                    team_color(not_bet_team),
                    '.)')
            ]);
        }
        return m('.card.stat-summary.h-100', [
            m('.card-body', [
                m('h5.card-title', 'Quick summary'),
                m('.card-text', [
                    m('ul', [
                        m('li', [length, ' matches ',
                            m('span.text-muted', 'over the last '),
                            duration.asHours().toFixed(), ' hours ',
                            m('span.text-muted', 'are shown on this page.')]),
                        m('li', [
                            m('span.text-muted', 'Total Gain/Loss: '),
                            display_gain_loss(total_gain), ' G',
                            ' (', display_gain_loss((total_gain / hours_shown) | 0), ' G/hour.)'
                        ]),
                        m('li',
                            m('span.text-muted', [
                                'Total Wagers: ', total_wager.toLocaleString(), ' G',
                                ' (', ((total_wager / hours_shown) | 0).toLocaleString(), ' G/hour.)'
                            ])
                        ),
                        m('li',
                            m('span.text-muted', [
                                'Accuracy is: ',
                                format_prediction(accuracy_average)
                            ])
                        ),
                        m('li',
                            m('span.text-muted', [
                                'Log Loss is: ',
                                average_log_loss.toFixed(4)
                            ])
                        ),
                        m('li', placed_bet_msg),
                    ]),
                    m('h5.my-1', 'Testimonial'),
                    m('img.mx-2.my-1', {
                        src: '/static.1/nacho-testimonial.jpeg',
                        height: '50px'
                    })
                ]),
            ])
        ])
    }
};

const BalanceLog = {
    view: function (vnode) {
        return m('table.table.mx-4', [
            m('thead', [
                m('tr', [
                    m('th', {scope: 'col'}, '#'),
                    m('th', {scope: 'col'}, 'Time'),
                    m('th.jr', {scope: 'col'}, 'Balance'),
                    m('th.jr', {scope: 'col'}, 'Gain/Loss'),
                    m('th.jr', {scope: 'col'}, 'Wager'),
                    m('th', {scope: 'col'}, 'Bet On'),
                    m('th', {scope: 'col'}, 'Winner'),
                    m('th', {scope: 'col'}, 'Win %'),
                    m('th.jr', {scope: 'col'}, 'Pool'),
                    m('th', {scope: 'col'}, 'Loser'),
                    m('th', {scope: 'col'}, 'Win %'),
                    m('th.jr', {scope: 'col'}, 'Pool'),
                ]),
            ]),
            m('tbody', State.balance_log.map(function (log) {
                const left_team = [
                    m('td', team_color(log.left_team)),
                    m('td', format_prediction(log.left_prediction)),
                    m('td.jr.text-muted', log.left_total_final.toLocaleString())
                ];
                const right_team = [
                    m('td', team_color(log.right_team)),
                    m('td', format_prediction(log.right_prediction)),
                    m('td.jr.text-muted', log.right_total_final.toLocaleString())
                ];
                const winner = log.left_wins ? left_team : right_team;
                const loser = log.left_wins ? right_team : left_team;

                return m('tr', [
                    m('th.text-muted', {scope: 'row'}, log.id),
                    m('td', log.time.format('LT')),
                    m('td.jr', log.new_balance.toLocaleString()),
                    m('td.jr', display_gain_loss(log.new_balance - log.old_balance)),
                    m('td.jr', log.wager.toLocaleString()),
                    m('td', team_color(log.bet_on)),
                    ...winner,
                    ...loser
                ]);
            }))
        ]);
    }
};

const BalanceChart = {
    view: function (vnode) {
        return m('.position-relative', m('canvas', {id: 'balance-chart'}));
    },
    onupdate: function (vnode) {
        const ctx = document.getElementById('balance-chart').getContext('2d');
        const yBalance = State.balance_log.map(log => log.new_balance).reverse();
        const X = State.balance_log.map(log => log.time.format('LT')).reverse();
        const chart = new Chart(ctx, {
            type: 'line',
            data: {
                labels: X,
                datasets: [{
                    label: 'Balance',
                    backgroundColor: '#303030',
                    borderColor: '#00bc8c',
                    pointRadius: 0.5,
                    data: yBalance,
                }],
            },
            options: {
                scales: {
                    xAxes: [{
                        ticks: {
                            autoSkipPadding: 10
                        },
                        gridLines: {
                            color: '#303030',
                        },
                    }],
                    yAxes: [{
                        ticks: {
                            callback: function (label, index, labels) {
                                return label.toLocaleString() + ' G';
                            }
                        },
                        gridLines: {
                            color: '#303030',
                        },
                    }]
                }
            }
        });
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
                m('.row.mb-4', [
                    m('.col', [
                        m(StatSummary),
                    ]),
                    m('.col.mt-4', [
                        m(BalanceChart),
                    ])
                ]),
                m('.row', [
                    m(BalanceLog)
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