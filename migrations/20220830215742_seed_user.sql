-- Add migration script here
INSERT INTO users (user_id, username, password_hash)
VALUES (
    'ddf8994f-d522-4659-8d02-c1d479057be6',
    'admin',
    '$argon2id$v=19$m=4096,t=3,p=1$EczPwijnyjlK2pf3IowQwQ$Qu+07OfGohBF/elBTReokUiwebLJ/wCRBxF2lAsh5Io'
);
