#this script will install dependencies and start the server
#need to run this script as root
#install chromedriver,whkhtmltopdf,openssl,rustup
sudo apt update && sudo apt upgrade -y;
apt install -y libssl-dev wkhtmltopdf chromium-driver git freeradius freeradius-postgresql curl postgresql llvm pkg-config gcc sudo --install-recommends;
#there is a need to setup freeradius
#for this some files need to be changed
# will document this before it is to late

#press enter
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh ; 

. "$HOME/.cargo/env";

git clone "https://github.com/n0vdd/smart_gest";

cd smart_gest;

cargo install sqlx-cli;
#setup the app user and db
#DATABASE_URL=postgres://appuser:apppassword@localhost:5432/appdb

sudo -u postgres psql -c "CREATE USER appuser WITH PASSWORD 'apppassword';"
sudo -u postgres psql -c "CREATE DATABASE appdb;"
sqlx database setup;

cargo r -r
