    public class userController{
        @autowired
        public userService userService;
            public void Getuser(int id){
        userService.getUserById(id);
    }
            public void Placeuser(user user){
        userService.Place(user);
    }
    }
