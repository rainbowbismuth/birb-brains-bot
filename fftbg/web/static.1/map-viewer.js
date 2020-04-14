
const MapState = {
    num: null,
    map: null,
    scene: null,
    camera: null,
    renderer: null,
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

function create_map_scene(vnode) {
    MapState.scene = new THREE.Scene();
    MapState.scene.background = new THREE.Color( 0x222222 );
    const aspect = window.innerWidth / window.innerHeight;
    const d = 10;
    MapState.camera = new THREE.OrthographicCamera( - d * aspect, d * aspect, d, - d, 1, 1000 );

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
            // if (tile.no_walk || tile.no_cursor) {
            //     continue;
            // }


            const big_height = (tile.height + tile.depth) + height;
            const big_geo = new THREE.BoxGeometry(1,big_height/2,1);
            const big_mat = new THREE.MeshLambertMaterial( {
                color: 0x888888,
                wireframe: false,
            } );
            const big_cube = new THREE.Mesh(big_geo, big_mat);
            big_cube.position.set(x,big_height/4,y);
            MapState.scene.add(big_cube);

            const geometry = new THREE.BoxGeometry(1,height,1);
            // const surface = tile.surface_type_numeric;
            // const color = 0xAAAAAA - (surface + (surface << 8) + (surface << 16)) * 2;
            const color = (cyrb53(tile.surface_type) & 0xFFFFFF) * 0.95;
            const material = new THREE.MeshLambertMaterial( {
                color,
                wireframe: false,
            } );
            const cube = new THREE.Mesh( geometry, material );
            cube.position.set(x,(big_height/2)+(height/2),y);
            MapState.scene.add(cube);

            // if (tile.slope_type !== 'Flat 0') {
            //     console.log(cube.position, height, tile.slope_type, {height: tile.height, slope: tile.slope_height});
            // }
        }
    }

    const spriteMap = new THREE.TextureLoader().load( "static.1/Ramza2-NW.gif" );
    spriteMap.wrapS = THREE.RepeatWrapping;
    spriteMap.repeat.x = - 1;
    const spriteMaterial = new THREE.SpriteMaterial( { map: spriteMap } );
    const sprite = new THREE.Sprite( spriteMaterial );

    // sprite.center = new THREE.Vector2(0.5,0);
    sprite.scale.set(0.5, 1, 1);
    sprite.position.set(5,(3+height)/2+height,0);
    MapState.scene.add( sprite );
    console.log(sprite);

    const axesHelper = new THREE.AxesHelper( 5 );
    axesHelper.position.set(MapState.map.width,2,-1);
    MapState.scene.add(axesHelper);
    MapState.camera.position.set(-d,d,-d);
    MapState.camera.lookAt(MapState.scene.position);
    MapState.camera.position.x += 10;
    MapState.camera.position.z += 10;

    vnode.dom.innerHTML = '';
    vnode.dom.appendChild(MapState.renderer.domElement);
    MapState.renderer.setPixelRatio( window.devicePixelRatio );
    setTimeout(() => {
        MapState.renderer.render(MapState.scene, MapState.camera);
    }, 1000);
}

const MapViewer = {
    view: function (vnode) {
        return m('div');
    },
    onupdate: function (vnode) {
        load_map(vnode.attrs.map_num, vnode);
    }
};