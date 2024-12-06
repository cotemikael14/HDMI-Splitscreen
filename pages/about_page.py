from tkinter import Toplevel, Label, Button

def show_about():
    """Affiche une fenêtre 'À propos'."""
    about_window = Toplevel()
    about_window.grab_set()
    about_window.overrideredirect(True)
    #about_window.overrideredirect(1)  # Supprime les bordures de fenêtre
    about_window.title("À propos")
    
    Label(about_window, text="Version 0.01", font=("Arial", 14)).pack(pady=10)
    Label(about_window, text="Date de la version : 2024-07-26", font=("Arial", 12)).pack(pady=10)
    Label(about_window, text="Créateur : Mikaël Côté", font=("Arial", 12)).pack(pady=10)
    
    Button(about_window, text="Fermer", command=about_window.destroy).pack(pady=10)
