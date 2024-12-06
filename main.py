from tkinter import Tk
from PIL import Image, ImageTk
from interface import create_interface
from scrip.config import check_and_create_config

def main():
    check_and_create_config()  # Vérifie et crée le fichier de configuration si nécessaire
    
    root = Tk()
    root.title("Interface de gestion des flux")
    root.geometry("900x700")
    
    # Définir l'icône de l'application
    try:
        root.iconbitmap('LOGO.ico')
    except Exception as e:
        print(f"Erreur lors du chargement de l'icône : {e}")
    
    create_interface(root)
    root.mainloop()

if __name__ == "__main__":
    main()
