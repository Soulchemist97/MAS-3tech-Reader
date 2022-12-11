# # Eigenschaften der Automaten Dateien

## https://github.com/Soulchemist97/MAS-3tech-Reader ##

## Benötigte Module laden ##
import os     # Anwählen von Ordnern
import io     # Arbeiten mit Filestreams
import pandas as pd  # Tabellen-Modul

from datetime import datetime as dt #Formatiert Datumswerte
import re #RegEx Sucht Patterns in Strings
from string import whitespace #Leerzeichen entfernen

import shutil # Kopieren, Verschieben in andere Ordner
try:
    from fpdf import FPDF #PDF Creator 
    # PDF-Dokumentation https://pyfpdf.readthedocs.io/en/latest/Tutorial/index.html
except:
    print("Fehlendes PDF-Modul: FPDF")



def create_Ordner(Ordner: str):
    """Checkt auf vorhandene Ordner und erstellt ggf. neuen Ordner

    Args:
        Ordner (str): Ordnerpfad zum neuen oder bereits existierenden Ordner

    Returns:
        Verzeichnis: Ordnerpfad als String
    """
    Verzeichnis = Ordner  # Verzeichnisse zusammensetzen ohne + "/"
    Vorhanden = os.path.isdir(Verzeichnis) #Prüfen ob Vorhanden

    if Vorhanden != True:
        os.makedirs(Verzeichnis) #Erstelle Verzeichnis
    else: 
        pass        
    
    return Verzeichnis


def DeleteEmptyFolder(Pfad: str):
    """
    Löscht leere UnterOrdner:
    1. Checkt ob Ordner existiert
    2. Checkt auf Unterordner
    3. Wenn Unterornder Leer sind, werden sie gelöscht

    Args:
        Pfad (str): Pfad des übergeordneten Ordners. Löscht alle leeren Unterordner
    """
    FolderExists = os.path.isdir(Pfad)
    if FolderExists:
        SubFolder = os.listdir(Pfad)

        for sub in SubFolder:
            SubFolder_Path = os.path.join(Pfad, sub)
            ##Skippen bei Dateien
            CheckForFile = os.path.isfile(SubFolder_Path)
            if CheckForFile:
                continue

            SubFolderExists = os.path.isdir(SubFolder_Path)
            FolderIsEmpty = True if len(os.listdir(SubFolder_Path)) == 0 else False  
            if SubFolderExists & FolderIsEmpty :
                shutil.rmtree(SubFolder_Path)
                print(SubFolder_Path,"Deleted")



Regex_Patterns = {
"Datum":r"\d{2}\.(0[1-9]|1[0-2])\.\d{4}",   #Langes Datum
"Date":r"\d{2}\.(0[1-9]|1[0-2])\.\d{2}",    #Kurzes Datum
"Uhrzeit":r"\d{2}:\d{2}:\d{2}",
"Zulassungsnummer":r"\d{9}",
"Ausdruck":r"(A|B) \d{3}",
"Geraetetyp":r"\x1bK\"",
"Ablaufdatum":r"\d{4}/\d{2}",
"Geld":r"\d+(.\d{1,2})"
}




def Extract_Value(Wort: str,Regex_Pattern: str,Lines):
    """
    Wort in Zeile suchen und aus dieser Zeile nach einem RegEx-Pattern den Wert erhalten
    """
    for i in Lines:
        a=re.search(Wort,i) #Suche Wort in Zeile
        if a != None: #Wenn WOrt gefunden
            Z = re.search(Regex_Pattern,i)
            if Z != None:
                return Z[0]


def Extract_OtherValue(Wort:str,Lines,Versatz: int =1,Remove_Spaces: bool =True):
    """Wort in Zeile suchen und aus anderer Zeile den Wert erhalten

    Args:
        Wort (str): Wort in bestimmter Zeile.
        Lines (list): Liste der verwendeten Zeilen. Der zu durchsuchende Text
        Versatz (int, optional): Zeilenentfernung von der Zeile per Wort. Standard = 1.
        Remove_Spaces (bool, optional): Leerzeichen entfernen oder nicht. Standard = True.

    Returns:
        Wert: Gibt Wert nach Regex-Pattern aus.
    """
    index=0 #Zeilenindex
    for i in Lines:
        a=re.search(Wort,i) #Suche Zeile mit Wort 
        if a != None: # Wenn Wort gefunden
            Z= re.sub(' +', ' ', Lines[index+Versatz]) #Zeile Versetzt um die gefundene ohne Multi-spaces
            if Z != None:           
                return Z
        index+=1 


class Rechnung():

    #Standardwerte falls nichts gefunden wurde
    Datum_Anfang, Datum_Ende = None, None
    Ausdruck_Nr = None
    Ablaufdatum = "--/--"

    Saldo_1 = None
    Saldo_2 = None
    Einsaetze = None
    Gewinne = None
    Zulassung = None
    Geraetetyp = None


    def __init__(self,Aufstellort,Pfad):
        """Erstellen eines Rechnungs Objektes

        Args:
            Aufstellort (str): Name des Aufstellortes bspw. der Bar.
            Pfad (Str or file-like Object): Dateipfad oder Objekt der open(...)-Funktion
        """
        self.Ort = Aufstellort

        if type(Pfad) != io.TextIOWrapper : # File-like Object 
            self.Pfad = Pfad
            Input_File = Pfad
            FileStreamCheck = False
        elif type(Pfad) == io.TextIOWrapper:
            self.Pfad=Pfad.name
            Input_File = Pfad
            FileStreamCheck = True
        
        self.old_name = os.path.split(self.Pfad)[-1] #Pfad.split("\\")[-1]
        _, self.FileExtension = os.path.splitext(self.Pfad)

        
        self.Auslesen(Input_File,FileStreamCheck)
        

    def Auslesen(self,fileInput,FileStream_Check=False): #Auslesen der Daten aus Datei, Auflisten der Werte und Ausgabe der Dateiname
        """
        Liest aus verschiedenen Dokumentstrukturen die Daten je Quittung aus mittels Regex Patterns.
        """
        
        if FileStream_Check == False: 
            ### Datei öffnen und Liste aus Zeilen ausgeben
            File_Obj = open(fileInput,"r",encoding='utf8', errors='ignore') # Öffnen der Datei
        else:
            File_Obj = fileInput
        
        File_Lines = File_Obj.readlines() # Liste aller Zeilen
        self.File_Lines = File_Lines

        ######### Parameter auslesen ###########
        
        self.Zulassung =  Extract_Value(Wort="ZULASSUNG",Regex_Pattern=Regex_Patterns["Zulassungsnummer"],Lines=File_Lines,)
        self.Ausdruck_Nr = Extract_Value(Wort="AUSDRUCK",Regex_Pattern=Regex_Patterns["Ausdruck"],Lines=File_Lines,)
        if self.Ausdruck_Nr == None:
            self.Ausdruck_Nr = Extract_Value(Wort="KOPIE",Regex_Pattern=Regex_Patterns["Ausdruck"],Lines=File_Lines,)
        self.Ablaufdatum = Extract_Value(Wort="ABLAUF",Regex_Pattern=Regex_Patterns["Ablaufdatum"],Lines=File_Lines,)

        # self.Geraetetyp = Extract_Value(Wort="BAUART",Regex_Pattern=,Lines=File_Lines,)

        
        def MoneyFloat(Wort="GEWINNE" ,Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines):
            """Search Money values and returns them as float object

            Args:
                Wort (str, optional): Wort in Zeile der Geldsumme. Defaults to "GEWINNE".
                Regex_Pattern (_type_, optional): Pattern für Geld. Defaults to Regex_Patterns["Geld"].
                Lines (_type_, optional): Textzeilen. Defaults to File_Lines.

            Returns:
                Money_Value (float): Formatted money value
            """
            Money = Extract_Value(Wort,Regex_Pattern,Lines,)
            if Money != None:
                Money_wo_spaces=Money.translate({ord(c): None for c in whitespace}) #Spaces entfernen
                Money = float(Money_wo_spaces.replace(",",".")) #Leerzeichen entfernen
                
                return Money

        # Regex SALDO (1) && SALDO (2) checken

        self.Saldo_1 =   MoneyFloat(Wort=" \(1",Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines,)
        self.Saldo_2 =   MoneyFloat(Wort=" \(2",Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines,) 
        self.Einsaetze = MoneyFloat(Wort="EINSAETZE",Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines,)
        self.Gewinne =   MoneyFloat(Wort="GEWINNE", Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines)
        self.Einwurf = MoneyFloat(Wort = "EINWURF",Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines)
        self.Auswurf = MoneyFloat(Wort = "AUSWURF",Regex_Pattern=Regex_Patterns["Geld"],Lines=File_Lines)
                

        ##################
        ##Datum beziehen##
        ##################
        
        try:
            EndDatum_string = Extract_OtherValue(Wort="KASSIERUNG VOM",Lines=File_Lines,Versatz=2)
            EndDatum_string = re.search(Regex_Patterns["Datum"],EndDatum_string)
            
            AnfangsDatum_String = Extract_OtherValue(Wort="LETZTE KASSIERUNG",Lines=File_Lines,Versatz=2)
            if AnfangsDatum_String == None:
                AnfangsDatum_String = Extract_OtherValue(Wort="INBETRIEBNAHME",Lines=File_Lines,Versatz=2)     

            AnfangsDatum_String = re.search(Regex_Patterns["Datum"],AnfangsDatum_String)

            self.Datum_Anfang,self.Datum_Ende = AnfangsDatum_String[0] , EndDatum_string[0]
        except:
            print(fileInput,"Datum konnte nicht ausgelesen werden")

        ### Datum ausgelesen ###

        self.Geraetetyp= Extract_OtherValue(Regex_Patterns["Geraetetyp"],Lines=File_Lines,Versatz=1)

        
        Dateiname = f"{self.Ort} [{self.Datum_Anfang}-{self.Datum_Ende}]({str(self.Ausdruck_Nr)})({str(self.Zulassung)}){self.FileExtension}"
        self.Dateiname=Dateiname

        if FileStream_Check == False:
            File_Obj.close() #Schließen der Datei

        return Dateiname

        
    def __str__(self):
        
        Str_Box = f"""{self.Dateiname} \n 
        Ablauf: {self.Ablaufdatum} \r 
        Saldo 1: {self.Saldo_1} € \r
        Saldo 2: {self.Saldo_2} € \r
        Einsaetze: {self.Einsaetze} € \r
        Gewinne: {self.Gewinne} € \r
        Einwurf: {self.Einwurf} € \r
        Auswurf: {self.Auswurf} € \r
        Geraetetyp: {self.Geraetetyp} \r
        """

        return Str_Box
    
    def __repr__(self):
        return self.Dateiname

    def open(self): #Datei öffnen
        os.startfile(self.Pfad)
        

    def pdf(self,Zeilen:int=100):
        """
        Erstellt PDF der ersten x Zeilen

        Args:
            Zeilen (int, optional): Zeilen des PDFs. Defaults to 100.
        """
        pdf = FPDF() #PDF-Klasse
        pdf.add_page() 
        pdf.set_font("Arial", size = 10) 
        
        lines = open(self.Pfad,"r",encoding='utf8', errors='ignore')
        
        ### Schneiden ###
        Strings = [L for L in lines]
        if Zeilen != 0:
            Strings = Strings[0:Zeilen]
        #------#

        for line in Strings: 
            pdf.cell(180, 4, txt = line, ln = 1, align = 'L')  # Width, height,
        PDF_Pfad = os.path.join(create_Ordner(r"pdf/" +self.Ort),self.Dateiname.replace(self.FileExtension,".pdf"))
        try:
            pdf.output(PDF_Pfad)  
            lines.close()
        except UnicodeEncodeError:
            print(self.Dateiname,": PDF konnte aufgrund von Sonderzeichen nicht erstellt werden")


 

class Aufstellort():    

    def Auflisten(self):
        """
        Listen für alle wichtigen Eigenschaften in der Datei erstellen
        """
        self.Dateinamen=[Rechnung.Dateiname for Rechnung in self.Rechnungen]
        self.Aufstellorte = [Rechnung.Ort for Rechnung in self.Rechnungen]
        self.Zulassungen = [Rechnung.Zulassung for Rechnung in self.Rechnungen]
        self.Ausdruck_Nummern = [Rechnung.Ausdruck_Nr for Rechnung in self.Rechnungen]
        self.Geraetetypen= [Rechnung.Geraetetyp for Rechnung in self.Rechnungen]
        self.Daten_Anfang= [Rechnung.Datum_Anfang for Rechnung in self.Rechnungen]
        self.Daten_Ende= [Rechnung.Datum_Ende for Rechnung in self.Rechnungen]
        self.Ablaufdaten= [Rechnung.Ablaufdatum for Rechnung in self.Rechnungen]
        self.Saldo_1_Liste= [Rechnung.Saldo_1 for Rechnung in self.Rechnungen]
        self.Saldo_2_Liste= [Rechnung.Saldo_2 for Rechnung in self.Rechnungen]
        self.Einsaetze = [Rechnung.Einsaetze for Rechnung in self.Rechnungen]
        self.Gewinne = [Rechnung.Gewinne for Rechnung in self.Rechnungen]
        self.Einwuerfe = [Rechnung.Einwurf for Rechnung in self.Rechnungen]
        self.Auswuerfe = [Rechnung.Auswurf for Rechnung in self.Rechnungen]

        #Dictionary für DataFrame
        self.dataset={"Aufstellort":self.Aufstellorte,"Ausdruck_Nr": self.Ausdruck_Nummern, 
                                "Zulassungsnummer": self.Zulassungen,"Geraetetyp":self.Geraetetypen,
                                "Anfangsdatum":self.Daten_Anfang,"Enddatum":self.Daten_Ende,
                                "Ablaufdatum":self.Ablaufdaten,"Saldo1":self.Saldo_1_Liste, "Saldo2":self.Saldo_2_Liste,
                                "Einsaetze": self.Einsaetze,"Gewinne": self.Gewinne,
                                "Einwurf" : self.Einwuerfe,"Auswurf" : self.Auswuerfe
                                }
    

    def dataframe(self):
        """
        Aufgelistete Eigenschaften als Panda Dataframe formattieren
        """
        self.df = pd.DataFrame(self.dataset)
        return self.df
    
    def __init__(self,Ort: str =None,Pfad: str =None):
        """Initialisiert Aufstellort mit vollständigem Pfad oder dem Namen im Ordner INPUT.

        Args:
            Ort (str, optional): Name des Ordners in  INPUT. Defaults to None.
            Pfad (str, optional): Ordnerpfad des Aufstellorts. Defaults to None.

        Raises:
            ValueError: Fehler, wenn keine der beiden Werte gesetzt wurde.
        """
        
        if Ort == None and Pfad == None:
            OrtError = "Ort in Input oder Ordnerpfad benötigt"
            raise ValueError(OrtError)
        
        if Pfad == None:
            self.Ort = Ort 
            self.Input = os.path.join(Input_dir,self.Ort)
        elif Pfad != None:
            self.Input=Pfad
            self.Ort = Pfad.split("\\")[-1]

        
        DeleteEmptyFolder(os.path.join(*self.Input.split("\\"))) #Löscht Unterordner wenn Leer

        SubFolder_exists = os.path.isdir(os.path.join(self.Input,"Kass-Daten"))
   
        if SubFolder_exists:
            self.Input = os.path.join(Input_dir,self.Ort,"Kass-Daten")

        self.Output = os.path.join(Output_dir,self.Ort) # Ausgabe-Ordner
        self.Dateien = os.listdir(self.Input) #Liste aller Dateien
        Pfade = [os.path.join(self.Input,Rechnung) for Rechnung in self.Dateien]
        
        ## Liste aller Klassen-Objekte ##
        self.Rechnungen = [Rechnung(self.Ort,Quittung) for Quittung in Pfade]
        
        self.Auflisten() #Listen aller Eigenschaften erstellen
        self.dataframe() #Dataframe erstellen
    
    
    def __str__(self): #Print-Befehlausgabe
        Quittungen_str_list =[str(Rechnung) for Rechnung in self.Rechnungen]
        Quittungen_str = ''.join(Quittungen_str_list) # Langer Kombinierter String

        return Quittungen_str
    
    def __len__(self): #Anzahl Rechnungen ausgeben
        return len(self.Rechnungen)
      
    def __repr__(self):
        return self.Ort

    def __iter__(self): #Ausgabe als Liste bzw. Iterierbarkeit
        return iter(self.Rechnungen)
      
    def Verschieben(self,Loeschen="n",remove="n"):
        
        create_Ordner(self.Output) #Ordner erstellen

        for Rechnung in self.Rechnungen:
            copy_from = Rechnung.Pfad # Alter Pfad
            copy_to = os.path.join(self.Output, Rechnung.old_name)
            Old_Name = copy_to
            New_Name = os.path.join(self.Output, Rechnung.Dateiname)
            
            try:
                if remove == "y":
                    shutil.move(copy_from, New_Name)  #Verschieben auf einmal
                    continue
                    
                shutil.copy(copy_from, copy_to) #Kopieren
                os.rename(Old_Name, New_Name) #Umbennenen

                if Loeschen == "y":   # Loeschen  der alten Datei An/Aus
                    os.remove(copy_from)
                else:
                    pass    # Wenn nicht, weitermachen
                       
            except FileExistsError:
                print(Rechnung,"doppelt")
                pass
            except OSError:
                print(Rechnung,"Parameterfehler")
                pass    
            except IndexError:
                print(Rechnung, "Index-Error")
                pass

    def pdf(self,cut=True,N_Zeilen:int=100):
        """
        Text Dateien der Quiitungen als PDF ausgeben
        Args:
            cut (bool, optional): Ab einer Zeile geschnitten oder vollständig umgewandelt. Defaults to True.
            N_Zeilen (int, optional): Anzahl Zeilen, bis zu der abgeschnitten wird. Defaults to 100.
        """
        for Quittung in self.Rechnungen:
            if cut == False or N_Zeilen == 0:
                Quittung.pdf(Zeilen=0)
            else: 
                Quittung.pdf(Zeilen=N_Zeilen)

    
    def Excel(self):
        """
        Wichtige Daten aus den Rechnungen für einen Aufstellort in einer Excel Datei ausgeben
        """
        self.df.to_excel(create_Ordner("Zusammenfassungen") + "/" + self.Ort + "_Zusammenfassung.xlsx",index=False)
        return self.df

    def store(self):
        """
        Exportiert Datensatz als binäres Format
        """
        import pickle
        create_Ordner("Database")

        filename = f"Database/{self.Ort}_Database.dbs"

        with open(filename,"wb") as Out_File:
            pickle.dump(self,Out_File)

        """
        with open("Aufstellort_Database","rb") as in_file:
            new_Loc = pickle.load(in_file)
        """


if __name__ == "__main__":

    print("Datei aus Input Ordner werden geladen")

    ## Ordnerpfade zum befüllen mit Quittungen ##
    Input_dir = create_Ordner("Input")  #"Input"
    Output_dir = create_Ordner("Output") # Output

    # Aufstellorte Indizieren
    Orte = os.listdir(Input_dir)
    Locations=[Aufstellort(Ort) for Ort in Orte]

    Bool_Dict = {
        "y":True,
        "Y":True,
        "Yes":True,
        "yes":True,
        "Ja":True,
        "ja":True,
        "N":False,
        "n":False,
        "Nein":False,
        "nein":False}

    Bool_Dict.setdefault("n")


    # Eingabeaufforderungen
    Print_Frage = input("Dateiinfos im Terminal ausgeben? (y): ")

    if Bool_Dict.get(Print_Frage):
        for Loc in Locations:
            print(Loc)

    Remove_Frage = input("Verschieben (y), Kopieren (c), Nichts (n) : ")
    Excel_Frage = Bool_Dict.get(input("Save Excel Files? y:  "))
    PDF_Frage = Bool_Dict.get(input("Save as PDF? (y) :  "))

    if PDF_Frage:
        PDF_Zeilen = int(input("Anzahl Zeilen der PDFs? (0 = Alle): "))

    for Location in Locations:
        if PDF_Frage:
            Location.pdf(cut=True,N_Zeilen=PDF_Zeilen)
        if Remove_Frage == "y" or Remove_Frage == "c": 
         Location.Verschieben(remove=Remove_Frage)
        if Excel_Frage :
            Location.Excel()

    ### Delete Empty Folders ###
    DeleteEmptyFolder(Input_dir)
    DeleteEmptyFolder(Output_dir)
    DeleteEmptyFolder("pdf")