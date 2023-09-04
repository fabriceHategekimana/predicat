command! Deploy !deploy_predicat
command! Reset !rm data.db
command! Db term nu -c "open data.db"
command! Facts term nu -c "open /home/fabrice/sessions/projet/predicat/predicat/data.db | get facts"
command! Rules term nu -c "open /home/fabrice/sessions/projet/predicat/predicat/data.db | get rules"


