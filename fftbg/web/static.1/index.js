'use strict';

const State = {
    balance_log: [],
    placed_bet: null,
    team_summary: null,
};

function load_balance_log() {
    m.request({
        method: 'GET',
        url: '/balance-log-stats',
    }).then(function (result) {
        State.balance_log = result;
        State.balance_log.forEach(function (log) {
            log.time = moment(log.time + 'Z');
        });
        State.balance_log.sort((a, b) => b.id - a.id);
    });
}

function load_placed_bet() {
    m.request({
        method: 'GET',
        url: '/placed-bet',
    }).then(function (result) {
        State.placed_bet = result;
        State.placed_bet.time = moment(result.time + 'Z');
    });
}

function load_team_summary() {
    m.request({
        method: 'GET',
        url: '/team-summary'
    }).then(function (result) {
        State.team_summary = result;
    });
}

function load_summary_info() {
    load_balance_log();
    load_placed_bet();
    load_team_summary();
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

function team_color(team) {
    return m('span.just-right.color-' + team, team);
}

function format_prediction(percent) {
    return (percent * 100).toFixed(1) + '%';
}

const cyrb53 = function (str, seed = 0) {
    let h1 = 0xdeadbeef ^ seed, h2 = 0x41c6ce57 ^ seed;
    for (let i = 0, ch; i < str.length; i++) {
        ch = str.charCodeAt(i);
        h1 = Math.imul(h1 ^ ch, 2654435761);
        h2 = Math.imul(h2 ^ ch, 1597334677);
    }
    h1 = Math.imul(h1 ^ h1 >>> 16, 2246822507) ^ Math.imul(h2 ^ h2 >>> 13, 3266489909);
    h2 = Math.imul(h2 ^ h2 >>> 16, 2246822507) ^ Math.imul(h1 ^ h1 >>> 13, 3266489909);
    return 4294967296 * (2097151 & h2) + (h1 >>> 0);
};

const PlacedBet = {
    view: function (vnode) {
        if (State.balance_log.length === 0) {
            return '';
        }
        const placed_bet = State.placed_bet;
        const newest = State.balance_log[0].time;
        let placed_bet_msg = m('div', [
            'Betting is currently open!'
        ]);
        if (placed_bet && newest.isBefore(placed_bet.time)) {
            // const win_prediction = (cyrb53(placed_bet.id.toString()) % 1000 / 1000);
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
        return placed_bet_msg;
    }
};

const StatSummary = {
    view: function (vnode) {
        if (State.balance_log.length === 0) {
            return '';
        }
        const new_balance = State.balance_log[0].new_balance;
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
                                'Current balance is: ', new_balance.toLocaleString(), ' G'
                            ])),
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
                        m('li', m(PlacedBet)),
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

function label_gil(label, index, labels) {
    return label.toLocaleString() + ' G';
}

function label_percent(label, index, labels) {
    return (label * 100).toFixed() + '%';
}

function label_fixed(label, index, labels) {
    return label.toFixed(2);
}

function create_chart(vnode) {
    const ctx = document.getElementById('chart-' + vnode.attrs.attribute).getContext('2d');
    const y = State.balance_log.map(log => log[vnode.attrs.attribute]).reverse();
    const X = State.balance_log.map(log => log.time.format('LT')).reverse();
    vnode.state.chart = new Chart(ctx, {
        type: 'line',
        data: {
            labels: X,
            datasets: [{
                label: vnode.attrs.attribute,
                borderColor: '#00bc8c',
                pointRadius: 0.5,
                data: y,
            }],
        },
        options: {
            animation: {
                duration: 0,
            },
            hover: {
                animationDuration: 0,
            },
            responsiveAnimationDuration: 0,
            scales: {
                xAxes: [{
                    ticks: {
                        autoSkipPadding: 10,
                    },
                    gridLines: {
                        color: '#303030',
                    },
                }],
                yAxes: [{
                    ticks: {
                        callback: vnode.attrs.yLabel,
                        beginAtZero: true,
                        suggestedMax: 1.0
                    },
                    gridLines: {
                        color: '#303030',
                    },
                }]
            }
        }
    });
}

const SimpleChart = {
    view: function (vnode) {
        return m('.position-relative', m('canvas', {id: 'chart-' + vnode.attrs.attribute}));
    },
    oncreate: function (vnode) {
        create_chart(vnode);
    },
    onupdate: function (vnode) {
        if (vnode.state.chart) {
            vnode.state.chart.destroy();
        }
        create_chart(vnode);
    },
    onremove: function (vnode) {
        if (vnode.state.chart) {
            vnode.state.chart.destroy();
        }
    }
};

const NavBar = {
    view: function (vnode) {
        return m('ul.nav.navbar.navbar-expand-lg.navbar-dark.bg-dark', [
            m('li.navbar-brand', m(m.route.Link, {
                href: '/home',
            }, 'Birb Brains Bot')),
            m('li.nav-item', m(m.route.Link, {
                href: '/stats',
                class: 'nav-link'
            }, 'Stats')),
            m('span.navbar-text', [
                'A machine learning powered betting bot for ',
                m('a', {href: 'https://www.twitch.tv/fftbattleground'}, 'FFT Battleground')
            ])
        ]);

    }
};


const UnitPortrait = {
    view: function (vnode) {
        let unit;
        if (vnode.attrs.gender === 'Monster') {
            unit = vnode.attrs.job;
        } else {
            unit = vnode.attrs.job + vnode.attrs.gender[0];
        }
        let color = '';
        if (vnode.attrs.color !== 'champion' && vnode.attrs.gender !== 'Monster') {
            color = '_' + vnode.attrs.color[0].toUpperCase() + vnode.attrs.color.slice(1);
        }
        const cls = vnode.attrs.left ? '.portrait-left' : '.portrait-right';
        return m('img' + cls, {src: 'https://mustadio-images.s3.amazonaws.com/units/' + unit + color + '.gif'});
    }
};

const IndividualUnit = {
    view: function (vnode) {
        let pluses = vnode.attrs.plus.slice(0, 5).map(plus =>
            m('span.px-2.unit-summary-plus', [
                '▲ ',
                plus[0],
                ' +',
                format_prediction(plus[1]),
            ])
        );
        if (pluses.length === 0) {
            pluses = [m('span.px-2.unit-summary-plus', '~')];
        }
        let minuses = vnode.attrs.minus.slice(0, 3).map(minus =>
            m('span.px-2.unit-summary-minus', [
                '▼ ',
                minus[0],
                ' ',
                format_prediction(minus[1]),
            ])
        );
        if (minuses.length === 0) {
            minuses = [m('span.px-2.unit-summary-minus', '~')];
        }

        const header_cols = [
            m('th', {scope: 'col'}, 'Name'),
            m('th', {scope: 'col'}, 'Sign'),
            m('th', {scope: 'col'}, 'Brave'),
            m('th', {scope: 'col'}, 'Faith'),
        ];
        const tr1_cols = [
            m('td', vnode.attrs.name),
            m('td', vnode.attrs.sign),
            m('td', vnode.attrs.brave),
            m('td', vnode.attrs.faith),
        ];

        let header;
        let tr1;
        if (vnode.attrs.left) {
            header = m('tr', [
                m('th', {scope: 'col'}, ''),
                ...header_cols
            ]);
            tr1 = m('tr', [
                m('td', {rowspan: 3, style: {width: '70px'}}, m(UnitPortrait, vnode.attrs)),
                ...tr1_cols
            ]);
        } else {
            header = m('tr', [
                ...header_cols,
                m('th', {scope: 'col'}, ''),
            ]);
            tr1 = m('tr', [
                ...tr1_cols,
                m('td', {rowspan: 3, style: {width: '70px'}}, m(UnitPortrait, vnode.attrs)),
            ]);
        }

        return [
            m('thead' + ('.color-' + vnode.attrs.color), [header]),
            m('tbody', [
                tr1,
                m('tr', [
                    m('td', {colspan: 4}, m('.row.d-flex.flex-wrap.justify-content-center', pluses))
                ]),
                m('tr', [
                    m('td', {colspan: 4}, m('.row.d-flex.flex-wrap.justify-content-center', minuses))
                ]),
                m('tr', m(
                    'td',
                    {
                        colspan: 6,
                        style: {
                            height: '20px',
                            'border-style': 'none'
                        }
                    },
                    ''))
            ])
        ]
    }
};

const IndividualTeam = {
    view: function (vnode) {
        return [
            m('.row', m('.col', m('table.table.table-sm', [
                ...vnode.attrs.units.map(unit => m(IndividualUnit, {
                    left: vnode.attrs.left,
                    color: vnode.attrs.color,
                    ...unit
                }))]
            )))
        ]
    }
};

const TeamSummary = {
    view: function (vnode) {
        if (!State.team_summary) {
            return ''
        }
        const map_num = (/(\d+)/).exec(State.team_summary.map)[0];
        return [
            m('.row', m('.col-md', [
                m('h2.text-center', State.team_summary.map),
                m('h6.text-center', 'Use left and right arrow keys to rotate map.'),
                m('h6.text-center', {id: 'surface-type-display'}, 'Mouse over a surface to display the surface\'s type here.'),
                m(MapViewer, {map_num}),
            ])),
            m('.row', m('.col-md', m('h4.text-center', 'Team Summary'))),
            m('.row', [
                
                m('.col-md', m(IndividualTeam, {
                    'left': true,
                    'color': State.team_summary.left_team,
                    'units': State.team_summary.left_team_units
                })),
                m('.col-md', m(IndividualTeam, {
                    'left': false,
                    'color': State.team_summary.right_team,
                    'units': State.team_summary.right_team_units
                })),
            ])
        ]
    }
};

const Home = {
    view: function (vnode) {
        return m('.container', [
            m('.row', [
                m('.col', [
                    m(NavBar),
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
                    m(SimpleChart, {attribute: 'new_balance', yLabel: label_gil, log: State.balance_log}),
                ])
            ]),
            m(TeamSummary),
            m('.row', [
                m(BalanceLog)
            ])
        ]);
    }
};

const Stats = {
    view: function (vnode) {
        return m('.container', [
            m('.row', [
                m('.col', [
                    m(NavBar),
                    m('br'),
                    m('p', [
                        'Some more statistics, right now just a rolling average of the bot\'s accuracy ',
                        'and log loss'
                    ])
                ]),
            ]),
            m('.row', [
                m('.col', [
                    m(SimpleChart, {attribute: 'new_balance', yLabel: label_gil, log: State.balance_log}),
                    m(SimpleChart, {attribute: 'rolling_accuracy', yLabel: label_percent, log: State.balance_log}),
                    m(SimpleChart, {attribute: 'rolling_log_loss', yLabel: label_fixed, log: State.balance_log}),
                ])
            ])
        ])
    }
};

const Stream = {
    view: function (vnode) {
        return m('.px-1', m(PlacedBet));
    }
};

m.route(document.body, '/home', {
    '/home': Home,
    '/stats': Stats,
    '/stream': Stream,
    '/map/:map_num': MapViewer,
});

load_summary_info();

document.addEventListener("visibilitychange", function () {
        if (document.visibilityState === 'visible') {
            load_summary_info();
            installInterval();
        } else {
            uninstallInterval();
        }
    }
);

function installInterval() {
    if (State.interval_id) {
        return;
    }

    State.interval_id = setInterval(function () {
        if (!State.placed_bet) {
            return;
        }
        const old_time = State.placed_bet.time;
        load_placed_bet();
        if (State.placed_bet.time.isAfter(old_time)) {
            load_balance_log();
        }
        if (!State.team_summary
            || State.placed_bet.left_team !== State.team_summary.left_team
            || State.placed_bet.right_team !== State.team_summary.right_team) {
            load_team_summary();
        }
    }, 5000);
}

function uninstallInterval() {
    if (!State.interval_id) {
        return;
    }

    clearInterval(State.interval_id);
    State.interval_id = null;
}

installInterval();