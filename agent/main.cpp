#include <cpr/api.h>
#include <cpr/cpr.h>
#include <iostream>

int main() {
    cpr::Response r = cpr::Get(cpr::Url{"http://www.localhost:8000/reg"});
    std::cout << r.text << std::endl;
    std::cout << r.status_code << std::endl; 
    
    return 0;
}
