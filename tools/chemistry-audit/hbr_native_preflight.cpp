// Read-only native preflight; link the already-built IPhreeqc static library.
// Run from the worktree root. No recorded experiment answers or fit targets.
#include "IPhreeqc.h"
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <iterator>
#include <sstream>
#include <stdexcept>
#include <string>

static std::string read(const char* path) {
    std::ifstream stream(path);
    if (!stream) throw std::runtime_error(std::string("cannot read ") + path);
    return {std::istreambuf_iterator<char>(stream), std::istreambuf_iterator<char>()};
}

static double value(int id, int column) {
    int type = 0;
    double result = 0;
    char buffer[256] = {};
    if (GetSelectedOutputValue2(id, GetSelectedOutputRowCount(id) - 1, column,
                               &type, &result, buffer, sizeof(buffer)) != 0 || type != 3)
        throw std::runtime_error("selected output is not a native double");
    return result;
}

static int sealed_probe(const std::string& database) {
    const double native_r = 0.0820597, codata_r = 0.082057366;
    const double dose = 1e-3, temperature = 298.15;
    int checked = 0;
    for (double volume : {0.01, 0.1, 1.0}) {
        for (double input_r : {codata_r, native_r}) {
            const int id = CreateIPhreeqc();
            if (id < 0 || LoadDatabaseString(id, database.c_str()))
                throw std::runtime_error("cannot initialize sealed preflight engine");
            std::ostringstream input;
            input << std::setprecision(17)
                  << "SOLUTION 1\n temp 25\n units mol/kgw\n -water 0.1\n pH 7 charge\n"
                     "GAS_PHASE 1\n -fixed_volume\n -temperature 25\n -volume " << volume
                  << "\n HBr(g) " << dose * input_r * temperature / volume
                  << "\nSELECTED_OUTPUT\n -reset false\n -high_precision true\n"
                     "USER_PUNCH\n -headings Br_moles gas_moles\n"
                     "10 PUNCH TOT(\"Br\")*TOT(\"water\"), GAS(\"HBr(g)\")\nEND\n";
            if (RunString(id, input.str().c_str()))
                throw std::runtime_error(GetErrorString(id));
            const double aqueous = value(id, 0), gas = value(id, 1), total = aqueous + gas;
            const double expected = dose * input_r / native_r;
            if (std::abs(total - expected) > dose * 1e-9)
                throw std::runtime_error("sealed inventory differs from initializer R-ratio prediction");
            if (input_r == native_r && std::abs(total - dose) > dose * 1e-9)
                throw std::runtime_error("native-R encoded gas dose was not conserved");
            if (input_r == codata_r && (dose - total) / dose < 20e-6)
                throw std::runtime_error("old-R encoding did not reproduce the observed loss");
            std::cout << std::setprecision(15) << "volume_L=" << volume
                      << " input_R=" << input_r << " aqueous_Br=" << aqueous
                      << " gas_HBr=" << gas << " total_Br=" << total
                      << " error_ppm=" << 1e6 * (total / dose - 1.0)
                      << " predicted_error_ppm=" << 1e6 * (input_r / native_r - 1.0) << '\n';
            DestroyIPhreeqc(id);
            ++checked;
        }
    }
    std::cout << "PASS: " << checked << " sealed states; native-R conversion conserves the intended dose\n";
    return 0;
}

int main(int argc, char** argv) {
    try {
        auto database = read("vendor/iphreeqc/database/wateq4f.dat");
        const auto extension = read("data/thermo/sander-2023-hbr.dat");
        const auto end = database.rfind("\nEND");
        database.insert(end == std::string::npos ? database.size() : end, "\n" + extension);
        if (argc == 2 && std::string(argv[1]) == "--sealed") return sealed_probe(database);
        double reference_log_k = 0;
        std::istringstream lines(extension);
        for (std::string line; std::getline(lines, line);) {
            std::istringstream fields(line);
            std::string key;
            if (fields >> key; key == "log_k") fields >> reference_log_k;
        }
        int checked = 0;
        for (const char* chemistry : {"pH 7 charge\n", "pH 2 charge\nCl 0.01\n", "pH 12 charge\nK 0.01\n"}) {
            for (double dose : {1e-6, 1e-4, 1e-3}) {
                const int id = CreateIPhreeqc();
                if (id < 0) throw std::runtime_error("CreateIPhreeqc failed");
                if (LoadDatabaseString(id, database.c_str()))
                    throw std::runtime_error(GetErrorString(id));
                std::ostringstream input;
                input << std::setprecision(17)
                      << "SOLUTION 1\n temp 25\n units mol/kgw\n -water 0.1\n"
                      << chemistry
                      << "EQUILIBRIUM_PHASES 1\n HBr(g) 0 " << dose
                      << "\nSELECTED_OUTPUT\n -reset false\n -high_precision true\n"
                         "USER_PUNCH\n -headings Br_moles gas_remaining logK\n"
                         "10 PUNCH TOT(\"Br\")*TOT(\"water\"), EQUI(\"HBr(g)\"), LA(\"H+\")+LA(\"Br-\")-SI(\"HBr(g)\")\nEND\n";
                if (RunString(id, input.str().c_str()))
                    throw std::runtime_error(GetErrorString(id));
                const double aqueous = value(id, 0), gas = value(id, 1), log_k = value(id, 2);
                if (std::abs(aqueous + gas - dose) > 1e-10 * dose + 1e-13)
                    throw std::runtime_error("finite bromide inventory not conserved");
                if (aqueous < dose * 0.999 || gas < -1e-12)
                    throw std::runtime_error("finite HBr gas did not dissolve into dilute water");
                if (std::abs(log_k - reference_log_k) > 1e-7)
                    throw std::runtime_error("native phase mass action differs from reviewed data");
                DestroyIPhreeqc(id);
                ++checked;
                std::cout << std::setprecision(14) << "dose=" << dose << " Br=" << aqueous
                          << " gas=" << gas << " logK=" << log_k << '\n';
            }
        }
        std::cout << "PASS: " << checked << " finite gas-dose/native mass-action states\n";
        return 0;
    } catch (const std::exception& error) {
        std::cerr << "FAIL: " << error.what() << '\n';
        return 1;
    }
}
