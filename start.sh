#this script will install dependencies and start the server
#need to run this script as root
#install chromedriver,whkhtmltopdf,openssl,rustup
sudo apt update && sudo apt upgrade -y;
apt install -y libssl-dev wkhtmltopdf chromium-driver git freeradius freeradius-postgresql curl postgresql llvm pkg-config gcc sudo neovim lld --install-recommends;
#there is a need to setup freeradius
#for this some files need to be changed
# will document this before it is to late

#press enter
# there some way to run with -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh ; 

. "$HOME/.cargo/env";

git clone "https://github.com/n0vdd/smart_gest";

cd smart_gest;

cargo install sqlx-cli;
#setup the app user and db
sudo -u postgres psql -c "CREATE USER appuser WITH PASSWORD 'apppassword';";
sudo -u postgres psql -c "CREATE DATABASE appdb;";
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE appdb TO appuser;";

sudo -u postgres psql -c "CREATE USER radius WITH PASSWORD 'radpass';"
sudo -u postgres psql -c "CREATE DATABASE radius;"
sudo -u postgres psql -c "GRANT ALL PRIVILEGES ON DATABASE radius TO radius;"


sqlx database setup;
cd radius;
sqlx database setup;
cd ..;

cargo r -r
