const OFFSETS_INCLINE = {
    'N': [0, 5],
    'E': [5, 4],
    'S': [4, 1],
    'W': [1, 0],
};

const OFFSET_CONVEX = {
    'NE': 5,
    'SE': 4,
    'SW': 1,
    'NW': 0
};

const OFFSETS_CONCAVE = {
    'SW': [0, 1, 4],
    'NW': [0, 1, 5],
    'NE': [0, 4, 5],
    'SE': [1, 4, 5]
}

const MapState = {
    num: null,
    map: null,
    scene: null,
    camera: null,
    renderer: null,
    raycaster: null,
    selected_tile: null,
    mouse: new THREE.Vector2(-100, -100),
    dispose_me: [],
    camera_pos: 0,
    units: [],
    loader: new THREE.TextureLoader()
};

function load_map(i, vnode) {
    if (MapState.num !== i) {
        MapState.num = i;
        m.request({
            method: 'GET',
            url: '/map/' + i
        }).then(function (result) {
            MapState.map = result;
            create_map_scene(vnode);
        });
    }
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

const D = 8;

function surface_type_color(surface_type) {
    return 0xFFFFFF - (cyrb53(surface_type) & 0x5F8F5F);
}

function add_layer_tile(tile, geometry, height, x, big_height, y) {
    if (tile.slope_type != null) {
        if (tile.slope_type.startsWith('Incline')) {
            const [offset1, offset2] = OFFSETS_INCLINE[tile.slope_type[tile.slope_type.length - 1]];
            geometry.vertices[offset1].y += (height * tile.slope_height * 2);
            geometry.vertices[offset2].y += (height * tile.slope_height * 2);
        } else if (tile.slope_type.startsWith('Convex')) {
            const offset = OFFSET_CONVEX[tile.slope_type.slice(tile.slope_type.length - 2)];
            geometry.vertices[offset].y += (height * tile.slope_height * 2);
        } else if (tile.slope_type.startsWith('Concave')) {
            const [offset1, offset2, offset3] = OFFSETS_CONCAVE[tile.slope_type.slice(tile.slope_type.length - 2)];
            geometry.vertices[offset1].y += (height * tile.slope_height * 2);
            geometry.vertices[offset2].y += (height * tile.slope_height * 2);
            geometry.vertices[offset3].y += (height * tile.slope_height * 2);
        }
    }
    geometry.verticesNeedUpdate = true;
    geometry.computeVertexNormals();
    const color = surface_type_color(tile.surface_type);
    const material = new THREE.MeshLambertMaterial({
        color,
        wireframe: false,
    });
    const cube = new THREE.Mesh(geometry, material);
    cube.fftbg_tile = tile;
    cube.position.set(x, (big_height / 2) + (height / 2), y);
    MapState.scene.add(cube);
    MapState.dispose_me.push(material, geometry);
}

function dispose_everything() {
    for (const item of MapState.dispose_me) {
        item.dispose();
    }
    MapState.dispose_me = [];
    MapState.scene = null;
    MapState.camera = null;
    MapState.camera_pos = 0;
    MapState.units = [];
}

const TEAM_COLORS = {
    'red': 0xE74C3C,
    'blue': 0x375A7F,
    'green': 0x00BC8C,
    'yellow': 0x39C12,
    'white': 0xC5BFBF,
    'black': 0x614675,
    'purple': 0x8B60D0,
    'brown': 0xF4A460,
    'champion': 0xFD7E14,
};

class Unit {
    constructor(x, y, color, facing, nw_texture, sw_texture) {
        this.x = x;
        this.y = y;
        this.color = color;
        this.facing = facing;
        this.facing_num = this.facing_to_num();
        this.nw_texture = nw_texture;
        this.sw_texture = sw_texture;
        this.nw_material = new THREE.SpriteMaterial({map: this.nw_texture});
        this.sw_material = new THREE.SpriteMaterial({map: this.sw_texture});

        this.cur_texture = null;
        this.cur_material = null;
        this.cur_sprite = null;

        this.render_pos = new THREE.Vector3();

        MapState.dispose_me.push(this.nw_texture, this.sw_texture,
            this.nw_material, this.sw_material);
    }

    facing_to_num() {
        if (this.facing === 'North') {
            return 0;
        } else if (this.facing === 'West') {
            return 1;
        } else if (this.facing === 'South') {
            return 2;
        } else if (this.facing === 'East') {
            return 3;
        }
    }

    flip() {
        this.cur_texture.repeat.set(-1, 1);
        this.cur_texture.offset.set(1, 0);
    }

    no_flip() {
        this.cur_texture.repeat.set(1, 1);
        this.cur_texture.offset.set(0, 0);
    }

    remove_from_scene() {
        if (this.cur_sprite) {
            MapState.scene.remove(this.cur_sprite);
            this.cur_sprite = null;
        }
    }

    set_render_position(x, y, z) {
        this.render_pos.x = x;
        this.render_pos.y = y;
        this.render_pos.z = z;
    }

    add_to_scene() {
        this.remove_from_scene();
        const rotation = (MapState.camera_pos + this.facing_num) % 4;
        if (rotation === 0) {
            this.cur_material = this.nw_material;
            this.cur_texture = this.nw_texture;
            this.flip();
        } else if (rotation === 1) {
            this.cur_material = this.sw_material;
            this.cur_texture = this.sw_texture;
            this.flip();
        } else if (rotation === 2) {
            this.cur_material = this.sw_material;
            this.cur_texture = this.sw_texture;
            this.no_flip();
        } else if (rotation === 3) {
            this.cur_material = this.nw_material;
            this.cur_texture = this.nw_texture;
            this.no_flip();
        }

        this.cur_sprite = new THREE.Sprite(this.cur_material);
        this.cur_sprite.position.copy(this.render_pos);

        const width = this.cur_texture.image.width;
        const height = this.cur_texture.image.height;
        if (height > width) {
            this.cur_sprite.scale.x = width / height;
        } else {
            this.cur_sprite.scale.y = height / width;
        }

        MapState.scene.add(this.cur_sprite);
    }
}

function create_map_scene(vnode) {
    dispose_everything();
    MapState.scene = new THREE.Scene();
    MapState.scene.background = new THREE.Color(0x222222);

    const dw = vnode.dom.clientWidth;
    const dh = vnode.dom.clientHeight;
    const aspect = dw / dh;
    MapState.camera = new THREE.OrthographicCamera(-D * aspect, D * aspect, D, -D, 0, 1000);

    if (MapState.renderer === null) {
        MapState.renderer = new THREE.WebGLRenderer({canvas: vnode.dom});
        MapState.renderer.setSize(dw, dh);
    }

    MapState.raycaster = new THREE.Raycaster();

    const light = new THREE.AmbientLight(0x606060); // soft white light
    MapState.scene.add(light);
    const directionalLight = new THREE.DirectionalLight(0x69bbbb, 0.5);
    directionalLight.position.set(10, 20, -10);
    MapState.scene.add(directionalLight);
    // const helper = new THREE.DirectionalLightHelper( directionalLight, 5, 0xFFFFFF );
    // MapState.scene.add(helper);

    const height = 0.25;
    for (let y = 0; y < MapState.map.height; y++) {
        for (let x = 0; x < MapState.map.width; x++) {
            const tile = MapState.map.lower[y][MapState.map.width - (x + 1)];
            if (tile.no_walk) {
                continue;
            }
            const big_height = (tile.height + tile.depth) + height;
            const big_geo = new THREE.BoxGeometry(1, big_height / 2, 1);
            let big_color = 0x888888;
            if (tile.no_walk) {
                big_color += 0x661111;
            }
            if (tile.no_cursor) {
                big_color += 0x111166;
            }
            const big_mat = new THREE.MeshLambertMaterial({
                color: big_color,
                wireframe: false,
                opacity: 0.2,
                transparent: true,
            });
            const big_cube = new THREE.Mesh(big_geo, big_mat);
            big_cube.position.set(x, big_height / 4, y);
            MapState.scene.add(big_cube);
            MapState.dispose_me.push(big_geo, big_mat);

            const geometry = new THREE.BoxGeometry(1, height, 1);
            add_layer_tile(tile, geometry, height, x, big_height, y);
        }
    }
    for (let y = 0; y < MapState.map.height; y++) {
        for (let x = 0; x < MapState.map.width; x++) {
            const tile = MapState.map.upper[y][MapState.map.width - (x + 1)];
            const lower_tile = MapState.map.lower[y][MapState.map.width - (x + 1)];
            const big_height = (tile.height + tile.depth) + height;
            const lower_height = (lower_tile.height + lower_tile.depth) + height;
            if (tile.height === 0 || tile.no_walk || lower_height >= big_height) {
                continue;
            }
            const geometry = new THREE.BoxGeometry(1, height, 1);
            add_layer_tile(tile, geometry, height, x, big_height, y);
        }
    }

    function loadSprite(url, callback) {
        MapState.loader.load(url, function (texture) {
            texture.generateMipmaps = false;
            texture.minFilter = THREE.NearestFilter;
            texture.wrapS = THREE.ClampToEdgeWrapping;
            texture.wrapT = THREE.ClampToEdgeWrapping;
            callback(texture);
        });
    }

    function load_unit(start) {
        let unit;
        let color;
        if (start.team === 'Player 1') {
            unit = State.team_summary.left_team_units[start.unit];
            color = State.team_summary.left_team;
        } else {
            unit = State.team_summary.right_team_units[start.unit];
            color = State.team_summary.right_team;
        }
        const job = unit.job.replace(" ", "");
        const gender = unit.gender;
        let path = "static.1/sprites/";
        if (gender === 'Monster') {
            path += job;
        } else {
            path += job + '1' + gender[0];
        }
        const nw_url = path + '-NW.gif';
        const sw_url = path + '-SW.gif';

        // console.log(start.x, start.y, color, start.facing, nw_url);

        loadSprite(nw_url, function (nw_texture) {
            loadSprite(sw_url, function (sw_texture) {
                let y = start.y;
                let x = start.x;
                let render_x = MapState.map.width - (x + 1);
                let tile;
                if (start.layer) {
                    tile = MapState.map.upper[y][x];
                } else {
                    tile = MapState.map.lower[y][x];
                }
                const big_height = (tile.height + tile.slope_height / 2) + height;
                const y_coord = (big_height / 2) + height * 2.75;
                let unit = new Unit(start.x, start.y, color,
                    start.facing, nw_texture, sw_texture);
                unit.set_render_position(render_x, y_coord, y);
                unit.add_to_scene();
                MapState.units.push(unit);
                requestAnimationFrame(animate);
            });
        });
    }

    for (let start of MapState.map.starting_locations) {
        load_unit(start);
    }

    // const axesHelper = new THREE.AxesHelper(5);
    // axesHelper.position.set(MapState.map.width+1, 0, 0);
    // MapState.scene.add(axesHelper);
    set_camera_pos(0);

    MapState.dispose_me.push(MapState.scene);
}

document.onkeydown = function (e) {
    if (MapState.renderer === null) {
        return;
    }
    if (e.key === "ArrowLeft") {
        set_camera_pos(-1);
        e.preventDefault();
    } else if (e.key === "ArrowRight") {
        set_camera_pos(1);
        e.preventDefault();
    }
};

function animate() {
    if (MapState.renderer == null) {
        return;
    }
    const canvas = MapState.renderer.domElement;
    const width = (canvas.clientWidth * window.devicePixelRatio) | 0;
    const height = (canvas.clientHeight * window.devicePixelRatio) | 0;
    const needResize = canvas.width !== width || canvas.height !== height;
    if (needResize) {
        MapState.renderer.setSize(width, height, false);
        MapState.camera.aspect = width / height;
        MapState.camera.updateProjectionMatrix();
    }

    MapState.renderer.render(MapState.scene, MapState.camera);
}

function ray_cast_for_surface_type() {
    if (MapState.renderer == null) {
        return;
    }
    MapState.raycaster.setFromCamera(MapState.mouse, MapState.camera);
    const intersects = MapState.raycaster.intersectObjects(MapState.scene.children);
    MapState.selected_tile = null;
    for (const match of intersects) {
        if (!match.object.fftbg_tile) {
            continue;
        }
        MapState.selected_tile = match.object.fftbg_tile;
        break;
    }

    const display = document.querySelector("#surface-type-display");
    if (display != null) {
        if (MapState.selected_tile != null) {
            const tile = MapState.selected_tile;
            const hex = (surface_type_color(tile.surface_type) | 0).toString(16);
            display.style.color = '#' + '000000'.substr(0, 6 - hex.length) + hex;
            display.textContent = `${tile.surface_type} (${tile.height + tile.slope_height / 2}h)`;
            if (tile.no_walk) {
                display.textContent += ' (No walk)';
            }
        } else {
            display.textContent = 'Mouse over a surface to display the surface\'s type here.';
        }
    }
}

function set_camera_pos(n) {
    if (MapState.renderer === null) {
        return;
    }
    const camera_positions = [
        [0, 0],
        [0, MapState.map.height],
        [MapState.map.width, MapState.map.height],
        [MapState.map.width, 0],
    ];

    MapState.camera_pos = (MapState.camera_pos + n + camera_positions.length) % camera_positions.length;
    const [x_offset, z_offset] = camera_positions[MapState.camera_pos];
    MapState.camera.position.set(x_offset, Math.PI * 4, z_offset);
    const map_center = new THREE.Vector3(MapState.map.width / 2, Math.PI * 2.5, MapState.map.height / 2);
    MapState.camera.lookAt(map_center);
    MapState.camera.position.y -= 4;

    for (let unit of MapState.units) {
        unit.add_to_scene();
    }
    requestAnimationFrame(animate);
}

const MapViewer = {
    view: function (vnode) {
        return m('canvas.map-renderer');
    },
    oncreate: function (vnode) {
        vnode.dom.addEventListener('mousemove', (e) => {
            const canvas = vnode.dom;
            const rect = canvas.getBoundingClientRect();
            const width = canvas.clientWidth;
            const height = canvas.clientHeight;
            const x = e.clientX - rect.left;
            const y = e.clientY - rect.top;
            MapState.mouse.x = (x / width) * 2 - 1;
            MapState.mouse.y = -(y / height) * 2 + 1;
            ray_cast_for_surface_type();
            e.preventDefault();
        });
        load_map(vnode.attrs.map_num, vnode);
    },
    onupdate: function (vnode) {
        load_map(vnode.attrs.map_num, vnode);
    },
    onbeforeremove: function (vnode) {
        MapState.num = null;
        MapState.map = null;
        dispose_everything();
        MapState.renderer.dispose();
        MapState.renderer = null;
    }
};