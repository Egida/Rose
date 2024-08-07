#include <cpr/api.h>
#include <cpr/cpr.h>
#include <iostream>

const char USER_AGENT[] = "Opera/9.80 (Macintosh; Intel Mac OS X; U; en) Presto/2.2.15 Version/10.00";

int main() {
    cpr::Response r = cpr::Post(cpr::Url{"http://localhost:8000/reg"},
            cpr::Header{{"user-agent", USER_AGENT}});

    std::cout << r.text << std::endl;
    std::cout << r.status_code << std::endl; 
    
    return 0;
}
