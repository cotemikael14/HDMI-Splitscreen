import tkinter as tk
from pages.home_page import create_home_page
from pages.settings_page import create_settings_page
from pages.about_page import show_about

def create_interface(root):
    fullscreen = [False]

    def toggle_fullscreen(event=None):
        fullscreen[0] = not fullscreen[0]
        root.attributes("-fullscreen", fullscreen[0])
        update_menu_text()

    def exit_fullscreen(event=None):
        """Sortir du mode plein écran."""
        if fullscreen[0]:
            toggle_fullscreen()

    def update_menu_text():
        if fullscreen[0]:
            view_menu.entryconfig(0, label="Quitter le mode plein écran")
            root.config(menu="")
        else:
            view_menu.entryconfig(0, label="Plein écran")
            root.config(menu=menu_bar)

    def show_menu_bar(event=None):
        if fullscreen[0]:
            root.config(menu=menu_bar)

    def hide_menu_bar(event=None):
        if fullscreen[0]:
            root.config(menu="")

    def show_settings(refresh_streams_callback):
        create_settings_page(root, refresh_streams_callback)

    menu_bar, view_menu = create_home_page(
        root, 
        lambda refresh_callback: show_settings(refresh_callback), 
        toggle_fullscreen, 
        show_about
    )
    root.config(menu=menu_bar)

    root.bind("<F11>", toggle_fullscreen)
    root.bind("<Escape>", exit_fullscreen)
    root.bind("<Motion>", lambda event: show_menu_bar() if event.y < 50 else hide_menu_bar())
