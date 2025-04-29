from MAS_Reader import *
from GUI.form import Ui_MainWindow

import sys
from PyQt5 import QtWidgets
from PyQt5.QtGui import QIcon

app = QtWidgets.QApplication(sys.argv)

window = QtWidgets.QMainWindow()
window.setWindowTitle("MAS Reader")
window.setWindowIcon(QIcon("Logo\MAS_Reader_Logo.png"))

ui_window = Ui_MainWindow()
ui_window.setupUi(window)

Print_Frage = "n"
Remove_Frage = "n"
Excel_Frage = "n"
PDF_Frage = "n"


Dict_Bool = {True: "y", False:"n"}


def Indexing():


    if Print_Frage == "y":
        for Loc in Locations:
            print(Loc)

    # ui_window.treeWidget

    for Location in Locations:
        # Location.Rechnungen

        if PDF_Frage == "y":
            Location.pdf(cut=True)

        if Remove_Frage == "y" or Remove_Frage == "c": 
         Location.Verschieben(remove=Remove_Frage)




def Start_Button_Clicked():
    """
    Wird ausgeführt, wenn <START> gedrückt wird.
    """

    print(ui_window.PathBox.toPlainText())

    Input_dir = ui_window.PathBox.toPlainText()
    create_Ordner(Input_dir)

    Output_dir = ui_window.OutputBox.toPlainText()
    create_Ordner(Output_dir)

    # Aufstellorte Indizieren
    Orte = os.listdir(Input_dir)
    Locations=[Aufstellort(Pfad=os.path.join(Input_dir,Ort)) for Ort in Orte]


    for Location in Locations:

        if ui_window.PrintBox.isChecked():
            print(Location)

        if ui_window.ExcelBox.isChecked():
            Location.Excel()
            print("Convert to Excel")

        if ui_window.PDFBox.isChecked():
            print("Convert to PDF")
            Location.pdf(cut=True,N_Zeilen=100)
        

        if ui_window.IndexButton.isChecked():
            print("Datei werden Indiziert")
            Location.Verschieben(remove="n")
        elif ui_window.CopyButton.isChecked():
            print("Kopieren der Dateien")
            Location.Verschieben(remove="c")
        elif ui_window.RemoveButton.isChecked():
            print(f"Verschieben der Dateien in {Output_dir}")
            Location.Verschieben(remove="y")


ui_window.StartButton.setIcon(QIcon("Logo\MAS_Reader_Logo.png")) #Logo zum Knopf hinzufügen

ui_window.StartButton.clicked.connect(Start_Button_Clicked)


Output_dir = "Output"
create_Ordner(Output_dir)


if __name__ =="__main__":
    window.show()
    sys.exit(app.exec())
