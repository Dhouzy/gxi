# EditView

The EditView GTK widget, backed by Xi. Below is a screenshot of it being used in [gxi](https://github.com/Cogitri/gxi).

![screenshot](../../data/screenshot.png?raw=true)

## Contributing

Please see the docs on https://gxi.cogitri.dev/docs to learn more about gxi's inner workings. 
[gtk-rs' site](https://gtk-rs.org/) offers documentation and examples about how gtk-rs works.

Visit [Weblate](https://hosted.weblate.org/engage/gxi/) to translate gxi and its components.

## Native Dependencies
	* Cairo >= 1.16
	* GLib-2.0 >= 2.42
	* GTK+3>= 3.20
	* Pango >= 1.38
	* Rust >= 1.31


## Using this in your application

Right now EditView is being reworked to make it easier to use in your application (read: remove gxi specific parts;
make it more generic), so please refrain from using it just yet.