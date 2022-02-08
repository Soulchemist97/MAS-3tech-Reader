from MAS_Reader import *
from GUI.form import Ui_MainWindow

import sys
from PyQt5 import QtWidgets

app = QtWidgets.QApplication(sys.argv)

window = QtWidgets.QMainWindow()
window.setWindowTitle("MAS Reader")

ui_window = Ui_MainWindow()
ui_window.setupUi(window)

Print_Frage = "n"
Remove_Frage = "n"
Excel_Frage = "n"
PDF_Frage = "n"


def Indexing():
    Input_dir = ui_window.PathBox.toPlainText()

    # Aufstellorte Indizieren
    Orte = os.listdir(Input_dir)
    Locations=[Aufstellort(Pfad=os.path.join(Input_dir,Ort)) for Ort in Orte]

    if Print_Frage == "y":
        for Loc in Locations:
            print(Loc)

    # ui_window.treeWidget

    for Location in Locations:
        # Location.Rechnungen

        if PDF_Frage == "y":
            Location.pdf(cut="Y")

        if Remove_Frage == "y" or Remove_Frage == "c": 
         Location.Verschieben(remove=Remove_Frage)
        if Excel_Frage == "y":
            Location.Excel()



def Start_Button_Clicked():

    print(ui_window.PathBox.toPlainText())

    if ui_window.PrintBox.isChecked():
        Print_Frage ="y"

    if ui_window.ExcelBox.isChecked():
        Excel_Frage = "y"
        print("Excel Converter")

    if ui_window.PDFBox.isChecked():
        Remove_Frage = "y"
        print("Convert to PDF")
    

    if ui_window.IndexButton.isChecked():
        Remove_Frage = "n"
        print("Indizieren")
    elif ui_window.CopyButton.isChecked():
        Remove_Frage = "c"
        print("Kopieren")
    elif ui_window.RemoveButton.isChecked():
        Remove_Frage = "y"
        print("Verschieben")

    Indexing()


ui_window.StartButton.clicked.connect(Start_Button_Clicked)



if __name__ =="__main__":
    window.show()
    sys.exit(app.exec())