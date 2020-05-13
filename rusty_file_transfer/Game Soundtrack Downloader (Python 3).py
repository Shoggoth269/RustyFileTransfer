from bs4 import BeautifulSoup
import requests

def getAllDownloadLinks(url):
    #print(url)
    page = requests.get(url)
    if page.status_code != 200:
        print("FAILED REQUEST AT GRABBING ALL LINKS")
        return None
    soup = BeautifulSoup(page.text, features="html.parser")

    links = []

    downloadButtons = soup.find_all(attrs={"class" : "playlistDownloadSong"})
    #print(downloadButtons)
    for button in downloadButtons:
        #print(button.find("a")["href"])
        links.append("https://downloads.khinsider.com/" + button.find("a")["href"])

    return links

def downloadFile(baseDirectory, url):
    try:
        from urllib import unquote
    except ImportError:
        from urllib.parse import unquote
    
    page = requests.get(url)
    if page.status_code != 200:
        print("FAILED REQUESTING FILE PAGE")
        return None
    soup = BeautifulSoup(page.text, features="html.parser")

    ##too lazy to rewrite the last for loop to do all of this so we get an extra one here to get the href links
    temp = soup.find_all(attrs={"class" : "songDownloadLink"})
    downloads = []
    for t in temp:
        downloads.append(t.findParent()["href"])

    for download in downloads:
        name = unquote(download.split("/")[-1:][0])
        r = requests.get(download, allow_redirects=True)
        open(baseDirectory + name, "wb").write(r.content)

def main():
    import os
    import time

    #this is where we create our folders for each album
    baseDirectory = "D:/Google Drive/Music/Game Soundtracks/"
    
    #open a file containing the URLs and album names
    #each line represents an album beginning with album name, a double comma delimiter, followed by the link to the album
    info = open("C:/Users/Shogg/Desktop/GameDownloadInfo.txt", "r")
    infoMap = []
    for line in info:
        if line[0] == "#":
            continue
        infoMap.append((line.split(",,")[0], line.split(",,")[1][:-1])) #stripping newline off of URL

    #we have a map of tuples, (album/folder name, link to album)
    #for each of these, make the folder and get our downloads
    for folder, link in infoMap:
        print(folder + "\t\t" + link)
        if not os.path.exists(baseDirectory + folder):
            os.makedirs(baseDirectory + folder)
        downloadLinks = getAllDownloadLinks(link)
        #print(downloadLinks)
        for downloadLink in downloadLinks:
            downloadFile(baseDirectory + folder + "/", downloadLink)
            time.sleep(5) #refrain from hitting the server too much
    
        
        

if __name__ == "__main__":
    main()
