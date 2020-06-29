# Use `GNOME Tweaks` instead
# Keyboard & Mouse -> 
#					Overview Shortcut (Right Super)
#					Additional Layout Options -> Ctrl position -> 
#											Swap Ctrl and Caps Lock
# 
# make right super - left super
#xmodmap -e "keycode 134 = Super_L NoSymbol Super_L"
# 
# swap casp with ctrl
#xmodmap -e "remove Lock = Caps_Lock"
#xmodmap -e "remove Control = Control_L"
#xmodmap -e "keysym Control_L = Caps_Lock"
#xmodmap -e "keysym Caps_Lock = Control_L"
#xmodmap -e "add Lock = Caps_Lock"
#xmodmap -e "add Control = Control_L"

abbr -a .. 'cd ..'
abbr -a ns nautilus
abbr -a e nvim
abbr -a gs 'git status'
abbr -a ga 'git add .'
abbr -a gc 'git commit -m'
abbr -a gp 'git push'
abbr -a lg 'git lg'
abbr -a cfg 'cd ~/.config/'
abbr -a cfe 'nvim ~/.config/nvim/init.vim'
abbr -a cff 'nvim ~/.config/fish/config.fish'

# make exa bindings if exists
if command -v exa > /dev/null
	abbr -a l 'exa'
	abbr -a ls 'exa'
	abbr -a la 'exa -a'
	abbr -a ll 'exa -l'
	abbr -a lll 'exa -la'
else
	abbr -a l 'ls'
	abbr -a ll 'ls -l'
	abbr -a lll 'ls -la'
end

# Type - to move up to top parent dir which is a repository
function d
	while test $PWD != "/"
		if test -d .git
			break
		end
		cd ..
	end
end

if test -f /home/orsenkucher/.autojump/share/autojump/autojump.fish;
	source /home/orsenkucher/.autojump/share/autojump/autojump.fish; 
end

