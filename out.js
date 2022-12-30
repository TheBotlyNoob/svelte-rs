
import {
    attr,detach,element,insert,SvelteComponent,init,safe_not_equal
} from "svelte/internal";

function create_fragment(ctx) {
    let h11;let html1;

    return {
        c() {
            
            
            

            html1 = element("html");
            
            

            h11 = element("h1");
            
            ;
        },
        m(target, anchor) {
            
                insert(target, h11, anchor);
            ;
        },
        p: noop,
        i: noop,
        o: noop,
        d(detaching) {
            if (detaching) {
                detach(h11);detach(html1);
            }
        }
    };
}

class App extends SvelteComponent {
    constructor(options) {
        super();
        init(this, options, null, create_fragment, safe_not_equal, {   });
    }
}

export default App;
        
