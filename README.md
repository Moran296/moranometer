# \*\~=Moranometer=~*

The Moranometer is a simple, easy-to-use, and powerful tool for
visualizing the distribution of a variable in a dataset.

No, I am kidding, Git copilot wrote this.

The moranometer is mainly a manager for trello board with several users and moderators managed from a telegram bot. For the moment, the main moderator is Moran, a **beast of extraordinary strength**.

Except managing trello, it will also supply other services and knowledge about the moderator, to create a work managing application and a cult of personality application, all-in-one.

Or in more simple words, the moranometer is a telegram bot for managing Moran's services.


## Using your own moranometer
Currently to use your own moranometer you need a telegram bot and trello account. You need to supply three environment variables:

 - TELOXIDE_MORANOMETER: telegram bot token
 - TRELLO_KEY: trello api key
 - TRELO_TOKEN: trello api token

Running the moranometer first time will create a 'users.json' file in the current directory. This is currently where you supply users with their permissions for boards.

for example:

    [
        {
            "name":"Moran",
            "id":1453903430,
            "boards":["TestBoard","Moranometer", "宝宝们"]
        }
    ]

Moran has access to three boards: TestBoard, Moranometer and 宝宝们.
Name and boards can be changed freely but notice that changing the id will cause the moranometer to lose access to the user.
Every new user contacting the moranometer bot will be added to the file (according to the user's id).



## TODO:
* Hash boards and boards id in a database instead of using  a json file.
* Seperate moderator to board moderator and admin and add field in user (and json) to indicate the permissions.
* Write documentation for using own moranometer.
