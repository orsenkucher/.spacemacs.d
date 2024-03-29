# Installation

## Emacs
download emacs from
```
https://www.gnu.org/software/emacs/download.html
```
and unzip to any directory

create shortcut to runemacs.exe (on Windows)

add *$HOME* environment variable pointing to your user home dir (also on Windows)
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
install go dependencies

### [new] use this
```
GO111MODULE=on go get -v golang.org/x/tools/gopls@latest
go get -u -v golang.org/x/tools/cmd/godoc
go get -u -v golang.org/x/tools/cmd/goimports
go get -u -v golang.org/x/tools/cmd/gorename
go get -u -v golang.org/x/tools/cmd/guru
go get -u -v github.com/cweill/gotests/...
go get -u -v github.com/davidrjenni/reftools/cmd/fillstruct
go get -u -v github.com/fatih/gomodifytags
go get -u -v github.com/godoctor/godoctor
go get -u -v github.com/golangci/golangci-lint/cmd/golangci-lint
go get -u -v github.com/haya14busa/gopkgs/cmd/gopkgs
go get -u -v github.com/josharian/impl
go get -u -v github.com/mdempsky/gocode
go get -u -v github.com/rogpeppe/godef
go get -u -v github.com/zmb3/gogetdoc
```

### [old] don't use
```
go get -u -v github.com/nsf/gocode
go get -u -v github.com/rogpeppe/godef
go get -u -v golang.org/x/tools/cmd/guru
go get -u -v golang.org/x/tools/cmd/gorename
go get -u -v golang.org/x/tools/cmd/goimports

go get -u -v github.com/alecthomas/gometalinter
gometalinter --install --update
```
gometalinter has faster modern alternatives - golangci-lint

## Flutter 
*I had to remove dart/flutter support from my .spacemacs*

You have to install *emacs27* with much faster json parser embedded into it for dart lsp to work properly 

Flutter folder have to be in *$HOME* dir

But to be honest, emacs flutter support is very poorly done compared to vscode

## Windows diff error
DiffUtils for Windows
```
http://gnuwin32.sourceforge.net/packages/diffutils.htm
```
don't forget to update your environment *$PATH* variable!

## Helm search
you may be lacking grep-like search, so
```
choco install ag
```
will help

## Good to remember
*toggle-input-method*
is used to write russian keystrokes on US keyboard

to download icon fonts first
`SPC SPC all-the-icons-install-fonts`
and install them manually (Windows)

## Uninstall
```
cd ~
rm .spacemacs
rm -rf .emacs.d
```
## git signing
[docs](https://help.github.com/en/github/authenticating-to-github/managing-commit-signature-verification)
