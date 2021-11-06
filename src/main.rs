use fltk::{image::*, app::*, browser::*, button::*, enums::*, input::*, prelude::*, window::*, frame::*, dialog::*, group::*};
use fltk_theme::{WidgetTheme, ThemeType};
use crate::Message::DisplayImage;

const APP_TITLE: &str = "Lucifer Tile Editor";
const COPYRIGHT: &str = "Copyright (C) 2021 Aurora Realms Entertainment";
const WIN_WIDTH: i32 = 800;
const WIN_HEIGHT: i32 = 560;

fn main() {
    let app = App::default();

    let widget_theme = WidgetTheme::new(ThemeType::Classic);
    widget_theme.apply();

    let (sender, receiver) = channel::<Message>();

    let mut win = create_main_window(sender.clone());

    let mut model = Model { sender: sender.clone(), dark_mode: false, image: None };

    while app.wait() {
        match receiver.recv() {
            Some(e) => {
                win(e.clone());
                match e {
                    Message::ChangeTheme => {
                        if !model.dark_mode {
                            let widget_theme = WidgetTheme::new(ThemeType::HighContrast);
                            widget_theme.apply();
                            model.dark_mode = true;
                        } else {
                            let widget_theme = WidgetTheme::new(ThemeType::Classic);
                            widget_theme.apply();
                            model.dark_mode = false;
                        }
                    }
                    Message::ClickOpenImage => {
                        model.load_png();

                        let image = model.clone().image;

                        if image.is_some() {
                            win(DisplayImage(image.unwrap()))
                        }
                    }

                    _ => println!("{:?}", e),
                }
            }
            None => {}
        }
    }
}

fn create_main_window(sender: Sender<Message>) -> Box<dyn FnMut(Message)> {
    let mut win = Window::default().with_size(WIN_WIDTH, WIN_HEIGHT).with_label(APP_TITLE);
    let mut flex = Flex::default().size_of_parent().column();

    win.make_resizable(true);

    let (mut top_pane_handler, mut top_pane) = create_top_pane(sender.clone());
    let mut main_pane_handler = create_main_pane(sender.clone());
    let (mut bottom_pane_handler, mut bottom_pane) = create_bottom_pane(sender.clone());
    let (mut footer_pane_handler, mut footer_pane) = create_footer_pane(sender.clone());

    flex.set_size(&mut footer_pane, 30);
    flex.set_size(&mut top_pane, 25);
    flex.set_size(&mut bottom_pane, 50);

    flex.end();
    win.show();
    win.end();

    Box::new(move |m| {
        top_pane_handler(m.clone());
        main_pane_handler(m.clone());
        bottom_pane_handler(m.clone());
        footer_pane_handler(m.clone());
        println!("{} {} {:?}", win.x(), win.y(), get_mouse());
    })
}

fn create_top_pane(sender: Sender<Message>) -> (Box<dyn FnMut(Message)>, Flex) {
    let flex = Flex::default().row();

    let mut btn = Button::default().with_label("Load PNG");
    btn.emit(sender.clone(), Message::ClickOpenImage);
    let btn = Button::default().with_label("Load Config");
    let btn = Button::default().with_label("Copy Config");
    let btn = Button::default().with_label("Copy ASM");
    let mut btn = Button::default().with_label("Theme");
    btn.emit(sender.clone(), Message::ChangeTheme);

    flex.end();
    (Box::new(|_| {}), flex)
}

fn create_footer_pane(sender: Sender<Message>) -> (Box<dyn FnMut(Message)>, Flex) {
    let flex = Flex::default().column();

    let _frame = Frame::default().with_label(COPYRIGHT);

    flex.end();
    (Box::new(|_| {}), flex)
}

fn create_main_pane(sender: Sender<Message>) -> Box<dyn FnMut(Message)> {
    let mut flex = Flex::default().column();
    let mut scroll = Scroll::default();
    scroll.end();
    flex.end();
    Box::new(move |m| {
        match m {
            DisplayImage(image) => {
                scroll.clear();
                scroll.begin();
                for r in 0..image.h() / 8 {
                    for c in 0..image.w() / 8 {
                        let mut btn = Button::default().with_size(33, 48).with_pos(c * 34, r * 49).with_label("?");
                        let mut image = get_tile_in_picture(r, c, &image);
                        image.scale(32, 32, true, true);
                        btn.set_image(Some(image));
                    }
                }
                scroll.end();
                scroll.redraw();
                flex.redraw();
            }
            _ => {}
        }
    })
}

fn create_bottom_pane(sender: Sender<Message>) -> (Box<dyn FnMut(Message)>, Flex) {
    let flex = Flex::default().column();

    let flex_a = Flex::default().row();

    let mut frame_cursor = Frame::default().with_label("0x00");
    let mut input_cursor = IntInput::default();
    let frame = Frame::default().with_label("Prefix: ");
    let mut input = Input::default();
    input.set_value("Tile_");

    flex_a.end();

    let flex_b = Flex::default().row();

    let btn = Button::default().with_label("+1 before");
    let btn = Button::default().with_label("+1 after");
    let btn = Button::default().with_label("-1 before");
    let btn = Button::default().with_label("-1 after");

    flex_b.end();

    flex.end();
    (Box::new(move |m| {
        if input_cursor.changed() {
            input_cursor.clear_changed();
            frame_cursor.set_label(&format!("{:#04x}", input_cursor.value().parse::<i32>().unwrap_or(0)));
        }
    }), flex)
}


#[derive(Clone, Debug)]
enum Message {
    ChangeTheme,
    ClickOpenImage,
    ImageLoaded,
    DisplayImage(PngImage),
}

#[derive(Clone, Debug)]
struct Model {
    sender: Sender<Message>,
    dark_mode: bool,
    image: Option<PngImage>,
}

impl Model {
    fn load_png(&mut self) {
        let path = file_chooser("Choose a picture", "*.png", "", false);
        match path {
            Some(path) => {
                self.image = PngImage::load(&path).ok();
                self.sender.send(Message::ImageLoaded);
            }
            None => return,
        };
    }
}

fn get_tile_in_picture(row: i32, col: i32, image: &PngImage) -> RgbImage {
    let original = image.to_rgb_data();
    let mut data = Vec::<u8>::new();
    for i in 0..8 {
        for j in 0..8 {
            data.push(original[((row * 8 + i) * image.w() * 4 + (col * 8 + j) * 4) as usize]);
            data.push(original[((row * 8 + i) * image.w() * 4 + (col * 8 + j) * 4 + 1) as usize]);
            data.push(original[((row * 8 + i) * image.w() * 4 + (col * 8 + j) * 4 + 2) as usize]);
            data.push(original[((row * 8 + i) * image.w() * 4 + (col * 8 + j) * 4 + 3) as usize]);
        }
    }
    // println!("data: {:?}", data);
    RgbImage::new(&data, 8, 8, ColorDepth::Rgba8).unwrap()
}

// use fltk::{image::*, app::*, browser::*, button::*, enums::*, input::*, prelude::*, window::*};
// use fltk_theme::{WidgetTheme, ThemeType};
// use fltk_flex::Flex;
// use fltk::frame::Frame;
// use fltk::dialog::{FileDialog, file_chooser};
// use fltk::group::Scroll;
//
// const APP_TITLE: &str = "Lucifer Tile Editor";
//
// #[derive(Clone, Debug)]
// enum Message {
//     ImageOpened,
//     Loop,
//     ThemeChanged,
//     ImageLoaded(String),
//     TileClicked(i32, i32),
//     CursorChangedTo(i32),
//     SetTileName(i32, i32, String),
// }
//
// fn main() {
//     let app = App::default();
//     let widget_theme = WidgetTheme::new(ThemeType::Classic);
//     widget_theme.apply();
//
//     let mut dark = false;
//
//     let (sender, reciever) = channel::<Message>();
//
//     let mut main_wind = create_main_window(sender);
//
//     let mut image: Option<PngImage> = None;
//
//     let mut cursor = 0;
//
//     while app.wait()
//     {
//         match reciever.recv() {
//             Some(Message::ImageOpened) => {
//                 let path = file_chooser("Choose a PNG picture.", "*.png", "", false).unwrap_or("".into());
//                 image = Some(PngImage::load(path.clone()).unwrap());
//                 main_wind(Message::ImageLoaded(path));
//             }
//             Some(Message::ThemeChanged) => {
//                 if !dark {
//                     let widget_theme = WidgetTheme::new(ThemeType::HighContrast);
//                     widget_theme.apply();
//                 } else {
//                     let widget_theme = WidgetTheme::new(ThemeType::Classic);
//                     widget_theme.apply();
//                 }
//                 dark = !dark;
//             }
//             Some(Message::CursorChangedTo(x)) => {
//                 cursor = x;
//             }
//             Some(Message::TileClicked(r, c)) => {
//                 main_wind(Message::SetTileName(r, c, format!("{:#04x}", cursor)));
//                 cursor += 1;
//                 main_wind(Message::CursorChangedTo(cursor));
//             }
//             Some(m) => {
//                 main_wind(m.clone());
//                 println!("{:?}", m)
//             }
//             None => main_wind(Message::Loop),
//         }
//     };
// }
//
// fn create_main_window(sender: Sender<Message>) -> Box<FnMut(Message)> {
//     let mut window = Window::default().with_size(800, 560).with_label(APP_TITLE);
//
//     let handler: Box<dyn FnMut(Message)>;
//
//     let mut container = Flex::default().size_of_parent().row();
//     let mut pad = Frame::default();
//     container.set_size(&mut pad, 10);
//     {
//         let mut container = Flex::default().column();
//
//         let mut pad = Frame::default();
//         container.set_size(&mut pad, 5);
//
//
//         let mut top_menu = create_top_menu(sender.clone(), &mut container);
//         let mut main_pane = create_main_pane(sender.clone());
//
//         handler = Box::new(move |m: Message| {
//             top_menu(m.clone());
//             main_pane(m.clone());
//         });
//
//         let mut footer = Frame::default().with_label("Copyright (C) 2021 Aurora Realms Entertainment");
//         container.set_size(&mut footer, 20);
//
//         container.end();
//     }
//     let mut pad = Frame::default();
//     container.set_size(&mut pad, 10);
//     container.end();
//
//     window.make_resizable(true);
//     window.end();
//     window.show();
//     handler
// }
//
// fn create_main_pane(sender: Sender<Message>) -> Box<dyn FnMut(Message)> {
//     let container = Flex::default().column();
//     let mut group = Scroll::default();
//     group.begin();
//
//     group.end();
//     container.end();
//
//     let mut n_cols = 0;
//
//     Box::new(move |m: Message| {
//         match m {
//             Message::ImageLoaded(s) => {
//                 println!("{}", s);
//                 let mut image = PngImage::load(s).unwrap();
//                 n_cols = image.w() / 8;
//                 println!("Info: w {} h {} depth {:?}", image.w(), image.h(), image.depth());
//                 // frame.set_image(Some(image.clone()));
//                 group.begin();
//                 for j in 0..image.h() / 8 {
//                     for i in 0..image.w() / 8 {
//                         let mut new = Button::default().with_size(34, 46).with_pos(i * 34, j * 46).with_label("??");
//                         new.set_label_color(Color::Magenta);
//                         new.set_label_size(10);
//                         let mut image = get_tile_in_picture(j, i, &image);
//                         image.scale(32, 32, true, true);
//                         new.set_image(Some(image));
//                         new.emit(sender.clone(), Message::TileClicked(j, i));
//                     }
//                 }
//                 group.end();
//                 group.redraw();
//                 group.scroll_to(0, 0);
//                 // println!("Data: {:?}", image.to_rgb_data())
//                 println!("Num of tiles: {} , cols: {}", group.children(), n_cols);
//                 println!("x {} y {}", group.child(19).unwrap().x(), group.child(19).unwrap().y());
//             }
//             Message::SetTileName(r, c, name) => {
//                 println!("Set {} {} to {}", r, c, name);
//                 group.child(n_cols * r + c).unwrap().set_label(&name);
//             }
//             _ => ()
//         }
//     })
// }
//
// fn create_top_menu(sender: Sender<Message>, outter: &mut Flex) -> Box<dyn FnMut(Message)> {
//     let mut label_cursor: Frame;
//     let mut cursor_input: IntInput;
//
//     let mut container = Flex::default().column();
//     {
//         let container = Flex::default().row();
//         let mut btn = Button::default().with_label("Load PNG");
//         btn.emit(sender.clone(), Message::ImageOpened);
//         let btn = Button::default().with_label("Copy ASM");
//         let btn = Button::default().with_label("Load Config");
//         let btn = Button::default().with_label("Copy Config");
//         container.end();
//     }
//     {
//         let container = Flex::default().row();
//         let btn = Button::default().with_label("+1 after cursor");
//         let btn = Button::default().with_label("-1 after cursor");
//         let btn = Button::default().with_label("+1 before cursor");
//         let btn = Button::default().with_label("-1 before cursor");
//         container.end();
//     }
//     {
//         let container = Flex::default().row();
//
//         label_cursor = Frame::default().with_label("Cursor");
//         cursor_input = IntInput::default();
//
//         let mut btn = Button::default().with_label("Theme");
//         btn.emit(sender.clone(), Message::ThemeChanged);
//         let btn = Button::default().with_label("Names");
//         container.end();
//     }
//     container.end();
//     outter.set_size(&mut container, 80);
//     Box::new(move |m: Message| {
//         let cursor = cursor_input.value().parse::<i32>().unwrap_or(0);
//         label_cursor.set_label(&format!("Cursor: {:#04x}", cursor));
//         if cursor_input.changed() {
//             sender.clone().send(Message::CursorChangedTo(cursor));
//             cursor_input.clear_changed();
//         }
//         match m {
//             Message::CursorChangedTo(x) => {
//                 cursor_input.set_value(&format!("{}", x));
//             }
//             _ => {}
//         }
//     })
// }
