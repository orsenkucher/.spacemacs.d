# Installation

## Emacs
download emacs from
```
https://www.gnu.org/software/emacs/download.html
```
and unzip to any directory
create shortcut to runemacs.exe (on Windows)

## Spacemacs
clone spacemacs repo to your home directory
```
$ git clone https://github.com/syl20bnr/spacemacs ~/.emacs.d
```
run `runemacs.exe` on Windows
or `emacs` not on Windows
```
$ cd ~/.emacs.d
$ git checkout develop
```
then run emacs again

## .spacemacs.d

clone this repository
```
$ git clone https://github.com/orsenkucher/.spacemacs.d ~/.spacemacs.d
```

run setup commands
```
$ cd ~/.spacemacs.d
$ sh ./setup.sh
```

## GO
Install go dependencies
```
go get -u -v github.com/nsf/gocode
go get -u -v github.com/rogpeppe/godef
go get -u -v golang.org/x/tools/cmd/guru
go get -u -v golang.org/x/tools/cmd/gorename
go get -u -v golang.org/x/tools/cmd/goimports

go get -u -v github.com/alecthomas/gometalinter
gometalinter --install --update
```

## Windows diff error
DiffUtils for Windows
```
http://gnuwin32.sourceforge.net/packages/diffutils.htm
```
Don't forget to update your environment PATH variable!

## Helm search
You may be lacking grep-like search, so
```
choco install ag
```
will help

## Good to remember
*toggle-input-method*
is used to write russian keystrokes on US keyboard
