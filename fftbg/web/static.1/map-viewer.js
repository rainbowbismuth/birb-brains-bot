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
    camera_pos: 0,
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

const cyrb53 = function(str, seed = 0) {
    let h1 = 0xdeadbeef ^ seed, h2 = 0x41c6ce57 ^ seed;
    for (let i = 0, ch; i < str.length; i++) {
        ch = str.charCodeAt(i);
        h1 = Math.imul(h1 ^ ch, 2654435761);
        h2 = Math.imul(h2 ^ ch, 1597334677);
    }
    h1 = Math.imul(h1 ^ h1>>>16, 2246822507) ^ Math.imul(h2 ^ h2>>>13, 3266489909);
    h2 = Math.imul(h2 ^ h2>>>16, 2246822507) ^ Math.imul(h1 ^ h1>>>13, 3266489909);
    return 4294967296 * (2097151 & h2) + (h1>>>0);
};

const D = 10;

function add_layer_tile(tile, geometry, height, x, big_height, y) {
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
    geometry.verticesNeedUpdate = true;
    geometry.computeVertexNormals();
    const color = (cyrb53(tile.surface_type) & 0xFFFFFF) * 0.95;
    const material = new THREE.MeshLambertMaterial({
        color,
        wireframe: false,
    });
    const cube = new THREE.Mesh(geometry, material);
    cube.position.set(x, (big_height / 2) + (height / 2), y);
    MapState.scene.add(cube);
}

function create_map_scene(vnode) {
    MapState.scene = new THREE.Scene();
    MapState.scene.background = new THREE.Color( 0x222222 );
    const aspect = window.innerWidth / window.innerHeight;
    MapState.camera = new THREE.OrthographicCamera( - D * aspect, D * aspect, D, - D, 1, 1000 );

    MapState.renderer = new THREE.WebGLRenderer();
    MapState.renderer.setSize(window.innerWidth*0.75, window.innerHeight*0.75);

    const light = new THREE.AmbientLight( 0x606060 ); // soft white light
    MapState.scene.add( light );
    const directionalLight = new THREE.DirectionalLight( 0x69bbbb, 0.5 );
    directionalLight.position.set(10,20,-10);
    MapState.scene.add( directionalLight );
    // const helper = new THREE.DirectionalLightHelper( directionalLight, 5, 0xFFFFFF );
    // MapState.scene.add(helper);
    const height = 0.25;
    for (let y = 0; y < MapState.map.height; y++) {
        for (let x = 0; x < MapState.map.width; x++) {
            const tile = MapState.map.lower[y][MapState.map.width-(x+1)];

            const big_height = (tile.height + tile.depth) + height;
            const big_geo = new THREE.BoxGeometry(1,big_height/2,1);
            let big_color = 0x888888;
            if (tile.no_walk) {
                big_color += 0x661111;
            }
            if (tile.no_cursor) {
                big_color += 0x111166;
            }
            const big_mat = new THREE.MeshLambertMaterial( {
                color: big_color,
                wireframe: true,
            } );
            const big_cube = new THREE.Mesh(big_geo, big_mat);
            big_cube.position.set(x,big_height/4,y);
            MapState.scene.add(big_cube);

            const geometry = new THREE.BoxGeometry(1,height,1);
            add_layer_tile(tile, geometry, height, x, big_height, y);

            // if (tile.slope_type !== 'Flat 0') {
            //     console.log(cube.position, height, tile.slope_type, {height: tile.height, slope: tile.slope_height});
            // }

        }
    }
    for (let y = 0; y < MapState.map.height; y++) {
        for (let x = 0; x < MapState.map.width; x++) {
            const tile = MapState.map.upper[y][MapState.map.width - (x + 1)];
            const big_height = (tile.height + tile.depth) + height;
            if (tile.height === 0) {
                continue;
            }
            const geometry = new THREE.BoxGeometry(1,height,1);
            add_layer_tile(tile, geometry, height, x, big_height, y);
        }
    }

    const spriteMap = new THREE.TextureLoader().load( "static.1/Ramza2-NW.gif" );
    spriteMap.wrapS = THREE.RepeatWrapping;
    spriteMap.repeat.x = - 1;
    const spriteMaterial = new THREE.SpriteMaterial( { map: spriteMap } );
    const sprite = new THREE.Sprite( spriteMaterial );

    // sprite.center = new THREE.Vector2(0.5,0);
    sprite.scale.set(0.5, 1, 1);
    const ramza_tile = MapState.map.lower[0][MapState.map.width-(5+1)];
    sprite.position.set(5,(ramza_tile.height+height+1+ramza_tile.slope_height/2)/2+height,0);
    MapState.scene.add( sprite );
    console.log(sprite);

    const axesHelper = new THREE.AxesHelper( 5 );
    axesHelper.position.set(MapState.map.width,2,-1);
    set_camera_pos(0);
    vnode.dom.innerHTML = '';
    vnode.dom.appendChild(MapState.renderer.domElement);
    MapState.renderer.setPixelRatio( window.devicePixelRatio );
    animate();
}

document.onkeydown = function(e) {
  if (e.key === "ArrowLeft") {
      set_camera_pos(-1);
  } else if (e.key === "ArrowRight") {
      set_camera_pos(1);
  }
};

function set_camera_pos(n) {
    const camera_positions = [
        [0, 0],
        [0, MapState.map.height],
        [MapState.map.width, MapState.map.height],
        [MapState.map.width, 0],
    ];

    MapState.camera_pos = (MapState.camera_pos + n + camera_positions.length) % camera_positions.length;
    const [x_offset, z_offset] = camera_positions[MapState.camera_pos];
    MapState.camera.position.set(x_offset,Math.PI*4,z_offset);
    const map_center = new THREE.Vector3(MapState.map.width/2, Math.PI*2, MapState.map.height/2);
    MapState.camera.lookAt(map_center);
}

function animate() {
    if (MapState.renderer != null) {
        requestAnimationFrame(animate);
        MapState.renderer.render(MapState.scene, MapState.camera);
    }
}

const MapViewer = {
    view: function (vnode) {
        return m('div');
    },
    onupdate: function (vnode) {
        load_map(vnode.attrs.map_num, vnode);
    },
    onremove: function (vnode) {
        MapState.num = null;
        MapState.map = null;
        MapState.scene.dispose();
        MapState.scene = null;
        MapState.renderer.dispose();
        MapState.renderer = null;
        MapState.camera = null;
        MapState.camera_pos = 0;
    }
};