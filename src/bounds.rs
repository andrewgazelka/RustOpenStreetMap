use crate::osm_parser::{Location, OpenStreetMap};
use crate::a_star::Path;

pub struct Bounds {
    pub from: Location,
    pub to: Location
}

pub trait Boundable {
    fn get_bounds(&self) -> Bounds;
}

impl Boundable for OpenStreetMap {
    fn get_bounds(&self) -> Bounds {
        let mut minx = f64::MAX;
        let mut miny = f64::MAX;
        let mut maxx = f64::MIN;
        let mut maxy = f64::MIN;

        for node in self.iterator() {
            let Location(x, y) = node.location;
            if x < minx {
                minx = x;
            }
            if x > maxx {
                maxx = x;
            }

            if y < miny {
                miny = y;
            }
            if y > maxy {
                maxy = y;
            }
        }

        Bounds {
            from: Location(minx, miny),
            to: Location(maxx, maxy)
        }
    }
}

impl <'a> Boundable for Path<'a> {
    fn get_bounds(&self) -> Bounds {
        let mut minx = f64::MAX;
        let mut miny = f64::MAX;
        let mut maxx = f64::MIN;
        let mut maxy = f64::MIN;

        for node in self.ids.iter().map(|&id| self.parent_map.get(id)) {
            let Location(x, y) = node.location;
            if x < minx {
                minx = x;
            }
            if x > maxx {
                maxx = x;
            }

            if y < miny {
                miny = y;
            }
            if y > maxy {
                maxy = y;
            }
        }

        Bounds {
            from: Location(minx, miny ),
            to: Location(maxx, maxy)
        }
    }
}
