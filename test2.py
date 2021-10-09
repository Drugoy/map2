import map2

reader = map2.Reader(patterns=[
    "/dev/input/by-path/pci-0000:03:00.0-usb-0:9:1.0-event-kbd"
])

writer = map2.Writer(reader)

print("start")

writer.map("pagedown", lambda: map2.exit())

tab_pressed = False
key_pressed = False


def tab_down():
    global tab_pressed, key_pressed
    tab_pressed = True
    key_pressed = False
writer.map("{tab down}", tab_down)


def tab_up():
    global tab_pressed, key_pressed
    tab_pressed = False

    if not key_pressed:
        writer.send("{tab}")
writer.map("{tab up}", tab_up)


def key_tab_mod(key):
    global tab_pressed
    if tab_pressed:
        writer.send("{alt down}{meta down}{shift down}" + key + "{alt up}{meta up}{shift up}");


caps_down = False

writer.map("^capslock", "capslock")


def capslock_down():
    global caps_down, key_pressed
    caps_down = True
    key_pressed = False
    writer.send("{ctrl down}")
writer.map("{capslock down}", capslock_down)


def capslock_up():
    global caps_down, key_pressed
    caps_down = False
    key_pressed = False
    writer.send("{ctrl up}")
    if not key_pressed:
        writer.send("{esc}")
writer.map("{capslock up}", capslock_up)

lalt = False
def leftalt_down():
    global lalt
    lalt = True
    writer.send_modifier("{leftalt down}")
writer.map("{leftalt down}", leftalt_down)

def leftalt_up():
    global lalt
    lalt = False
    writer.send_modifier("{leftalt up}")
writer.map("{leftalt up}", leftalt_up)


ralt = False
def rightalt_down():
    global ralt
    ralt = True
    writer.send_modifier("{rightalt down}")
writer.map("{rightalt down}", rightalt_down)

def rightalt_up():
    global ralt
    ralt = False
    writer.send_modifier("{rightalt up}")
writer.map("{rightalt up}", rightalt_up)


map2.wait()
