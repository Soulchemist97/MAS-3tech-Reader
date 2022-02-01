#!/usr/bin/env python
# coding: utf-8

# # Eigenschaften der Automaten Dateien

## Benötigte Module laden ##
import os   # Anwählen von Ordnern
import pandas as pd  # Tabellen-Modul

from datetime import datetime as dt #Formatiert Datumswerte
import re #RegEx Sucht Patterns in Strings
from string import whitespace #Leerzeichen entfernen

import shutil # Kopieren in Ordner
try:
    from fpdf import FPDF #PDF Creator 
    # PDF-Dokumentation https://pyfpdf.readthedocs.io/en/latest/Tutorial/index.html
except:
    print("Fehlendes PDF-Modul: FPDF")

#os.chdir(r"C:\Users\Janni\OneDrive\Python Stuff\Projekte\Automaten-Daten") #Definiere den Ordner


def create_Ordner(Ordner):
    """
    Checkt auf vorhandene Ordner und erstellt ggf. neuen Ordner
    """
    Verzeichnis = Ordner  # Verzeichnisse zusammensetzen ohne + "/"
    Vorhanden = os.path.isdir(Verzeichnis) #Prüfen ob Vorhanden

    if Vorhanden != True:
        os.makedirs(Verzeichnis) #Erstelle Verzeichnis
    else: 
        pass        
    
    return Verzeichnis


def DeleteEmptyFolder(Pfad):
    """
    Löscht Ordner, sofern Leer
    """
    FolderExists = os.path.isdir(Pfad)
    if FolderExists:
        SubFolder = os.listdir(Pfad)

        for sub in SubFolder:
            SubFolder_Path = os.path.join(Pfad, sub)
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
"Geld":r"(- |-| |-  |)(\d{6}|\d{5}|\d{4}|\d{3}|\d{2}|\d{1}),\d{2}"}



def Extract_Value(Wort,Regex_Pattern,Lines,print=True):
    """
    Wort in Zeile suchen und aus dieser Zeile nach einem RegEx-Pattern den Wert erhalten
    """
    for i in Lines:
        a=re.search(Wort,i) #Suche Wort in Zeile
        if a != None: #Wenn WOrt gefunden
            Z = re.search(Regex_Pattern,i)
            if Z != None:
                return Z[0]


def Extract_OtherValue(Wort,Lines,Versatz=1,Remove_Spaces=True):
    """
    Wort in Zeile suchen und aus anderer Zeile den Wert erhalten
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
        self.Ort = Aufstellort
        self.Pfad = Pfad #os.path.join(Pfad,Datei)
        self.Auslesen(self.Pfad)
        self.old_name = Pfad.split("\\")[-1]#Datei


    def Auslesen(self,filepath): #Auslesen der Daten aus Datei, Auflisten der Werte und Ausgabe der Dateiname
        """
        Liest aus verschiedenen Dokumentstrukturen die Daten je Quittung aus.
        Manche per Fester Zeilenangabe, andere per Wortsuche in gleicher Zeile.
        """
        
        ### Datei öffnen und Liste aus Zeilen asugeben
        fp = open(filepath,"r",encoding='utf8', errors='ignore') # Öffnen der Datei
        ap = fp.readlines(12000) # Liste aller Zeilen 12000
        self.ap = ap

        ######### Parameter auslesen ############
        
        self.Zulassung =  Extract_Value(Wort="ZULASSUNG",Regex_Pattern=Regex_Patterns["Zulassungsnummer"],Lines=ap,)
        self.Ausdruck_Nr = Extract_Value(Wort="AUSDRUCK",Regex_Pattern=Regex_Patterns["Ausdruck"],Lines=ap,)
        if self.Ausdruck_Nr == None:
            self.Ausdruck_Nr = Extract_Value(Wort="KOPIE",Regex_Pattern=Regex_Patterns["Ausdruck"],Lines=ap,)
        self.Ablaufdatum = Extract_Value(Wort="ABLAUF",Regex_Pattern=Regex_Patterns["Ablaufdatum"],Lines=ap,)

        # self.Geraetetyp = Extract_Value(Wort="BAUART",Regex_Pattern=,Lines=ap,)

        
        def MoneyFloat(Wort="GEWINNE" ,Regex_Pattern=Regex_Patterns["Geld"],Lines=ap):
            Money = Extract_Value(Wort,Regex_Pattern,Lines,)
            if Money != None:
                Money_wo_spaces=Money.translate({ord(c): None for c in whitespace}) #Spaces entfernen
                Money = float(Money_wo_spaces.replace(",",".")) #Leerzeichen entfernen
                
                return Money

        # Regex SALDO (1) && SALDO (2) checken

        self.Saldo_1 =   MoneyFloat(Wort=" \(1",Regex_Pattern=Regex_Patterns["Geld"],Lines=ap,)
        self.Saldo_2 =   MoneyFloat(Wort=" \(2",Regex_Pattern=Regex_Patterns["Geld"],Lines=ap,) 
        self.Einsaetze = MoneyFloat(Wort="EINSAETZE",Regex_Pattern=Regex_Patterns["Geld"],Lines=ap,)
        self.Gewinne =   MoneyFloat(Wort="GEWINNE" ,Regex_Pattern=Regex_Patterns["Geld"],Lines=ap)
                

        ##################
        ##Datum beziehen##
        ##################

        Daten = []
    
        for i in ap:
            Date = None    
            Long,Short = None,None
            Long=re.search(Regex_Patterns["Datum"],i)
            Short=re.search(Regex_Patterns["Date"],i)

            
            if Long != None:
                Date = dt.strptime(Long[0], "%d.%m.%Y") 
            elif Short != None:
                Date = dt.strptime(Short[0], "%d.%m.%y")

            if Date != None:
                Daten.append(Date.strftime("%d.%m.%Y"))    

        if len(Daten) >= 3:
            self.Datum_Anfang,self.Datum_Ende = Daten[2],Daten[1]
        else:
            pass

        ### Datum ausgelesen ###

        self.Geraetetyp= Extract_OtherValue(Regex_Patterns["Geraetetyp"],Lines=ap,Versatz=1)

        
        Dateiname = f"{self.Ort} [{self.Datum_Anfang}-{self.Datum_Ende}]({str(self.Ausdruck_Nr)})({str(self.Zulassung)}).ACK"
        self.Dateiname=Dateiname
        print(Dateiname)
        ### Nicht im Dateinamen
        print("Ablauf:",self.Ablaufdatum)
        print("Saldo 1:",self.Saldo_1,"€")
        print("Saldo 2:",self.Saldo_2,"€")
        print("Einsaetze:",self.Einsaetze,"€")
        print("Gewinne:",self.Gewinne,"€")
        print("Geraetetyp:",self.Geraetetyp)


        fp.close() #Schließen der Datei

        return Dateiname

        
    def __str__(self):
        return self.Dateiname
    
    def __repr__(self):
        return self.Dateiname
    
    def open(self): #Datei öffnen
        os.startfile(self.Pfad)
        
    def pdf(self):
        pdf = FPDF() #PDF-Klasse
        pdf.add_page() 
        pdf.set_font("Arial", size = 10) 
        
        lines = open(self.Pfad,"r",encoding='utf8', errors='ignore')
        
        for line in lines: 
            pdf.cell(180, 4, txt = line, ln = 1, align = 'L')  # Width, height,
        PDF_Pfad = os.path.join(create_Ordner(r"pdf/" +self.Ort),self.Dateiname.replace(".ACK",".pdf"))
        pdf.output(PDF_Pfad)  
        lines.close()

    def PDFcut(self):
        """
        Erstellt PDF der ersten 100 Zeilen
        """
        pdf = FPDF() #PDF-Klasse
        pdf.add_page() 
        pdf.set_font("Arial", size = 10) 
        
        lines = open(self.Pfad,"r",encoding='utf8', errors='ignore')
        
        Strings = [L for L in lines]
        Strings = Strings[0:100]

        for line in Strings: 
            pdf.cell(180, 4, txt = line, ln = 1, align = 'L')  # Width, height,
        PDF_Pfad = os.path.join(create_Ordner(r"pdf/" +self.Ort),self.Dateiname.replace(".ACK",".pdf"))
        pdf.output(PDF_Pfad)  
        lines.close()





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

        #Dictionary für DataFrame
        self.dataset={"Aufstellort":self.Aufstellorte,"Ausdruck_Nr": self.Ausdruck_Nummern, 
                                "Zulassungsnummer": self.Zulassungen,"Geraetetyp":self.Geraetetypen,
                                "Anfangsdatum":self.Daten_Anfang,"Enddatum":self.Daten_Ende,
                                "Ablaufdatum":self.Ablaufdaten,"Saldo1":self.Saldo_1_Liste, "Saldo2":self.Saldo_2_Liste,
                                "Einsaetze": self.Einsaetze,"Gewinne": self.Gewinne}
    

    def dataframe(self):
        """
        Aufgelistete Eigenschaften als Panda Dataframe formattieren
        """
        self.df = pd.DataFrame(self.dataset)
        return self.df
    
    def __init__(self,Ort):
        self.Ort = Ort 
        self.Input = os.path.join(Input_dir,Ort)

        # SubFolder_exists = os.path.isdir(os.path.join(Ort,"Kass-Daten"))
        # if SubFolder_exists:
            # self.Input = os.path.join(Input_dir,Ort,"Kass-Daten")

        self.Output = os.path.join(Output_dir,Ort) # Ausgabe-Ordner
        self.Dateien = os.listdir(self.Input) #Liste aller Dateien
        Pfade = [os.path.join(self.Input,Rechnung) for Rechnung in self.Dateien]
        
        ## Liste aller Klassen-Objekte ##
        self.Rechnungen = [Rechnung(Ort,Quittung) for Quittung in Pfade]
        
        self.Auflisten() #Listen aller Eigenschaften erstellen
        self.dataframe() #Dataframe erstellen
    
    
    def __str__(self): #Print-Befehlausgabe
        [print(Rechnung) for Rechnung in self.Rechnungen]
    
    def __len__(self): #Anzahl Rechnungen ausgeben
        return len(self.Rechnungen)
      
    def __repr__(self):
        return self.Ort
      
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

    def pdf(self,cut="Y"):
        """
        Text Dateien der Quiitungen als PDF ausgeben
        """
        for Quittung in self.Rechnungen:
            if cut == "Y":
                Quittung.PDFcut()
            elif cut != "Y":
                Quittung.pdf()
    
    def Excel(self):
        """
        Wichtige Daten aus den Rechnungen für einen Aufstellort in einer Excel Datei ausgeben
        """
        self.df.to_excel(create_Ordner("Zusammenfassungen") + "/" + self.Ort + "_Zusammenfassung.xlsx",index=False)
        return self.df

    def store(self):

        import pickle
        create_Ordner("Database")

        filename = f"Database/{self.Ort}_Database.dbs"

        with open(filename,"wb") as Out_File:
            pickle.dump(self,Out_File)

        """
        with open("BarDaRosa_Database","rb") as in_file:
            new_Loc = pickle.load(in_file)
        """


"""
Ordnerpfade zum befüllen mit Quittungen
"""

Input_dir = create_Ordner("Input")  #"Input"
Output_dir = create_Ordner("Output") # Output


if __name__ == "__main__":

    # Aufstellorte Indizieren
    Orte = os.listdir(Input_dir)
    Locations=[Aufstellort(Ort) for Ort in Orte]
    
    # Eingabeaufforderungen
    Remove_Frage = input("Verschieben (y), Kopieren (c), Nichts (n) : ")
    Excel_Frage = input("Save Excel Files? y:  ")
    PDF_Frage = input("Save as PDF? (y) :  ")
    Store_Frage = input("Save Database? (y) :  ")

    for Location in Locations:
        # Location.Rechnungen
        if PDF_Frage == "y":
            Location.pdf(cut="Y")

        if Remove_Frage == "y" or Remove_Frage == "c": 
         Location.Verschieben(remove=Remove_Frage)
        if Excel_Frage == "y":
            Location.Excel()
        if Store_Frage == "y":
            Location.store()

    ### Delete Empty Folders ###
    DeleteEmptyFolder(Input_dir)
    DeleteEmptyFolder(Output_dir)
    DeleteEmptyFolder("pdf")