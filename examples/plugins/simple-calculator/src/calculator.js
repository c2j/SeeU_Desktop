/**
 * Simple Calculator Plugin for iTools
 * 
 * This plugin provides basic mathematical calculation and unit conversion capabilities.
 */

class SimpleCalculator {
    constructor() {
        this.name = "Simple Calculator";
        this.version = "1.0.0";
        
        // Unit conversion factors (to base units)
        this.unitFactors = {
            // Length (to meters)
            'm': 1,
            'km': 1000,
            'cm': 0.01,
            'mm': 0.001,
            'ft': 0.3048,
            'in': 0.0254,
            'mile': 1609.34,
            'yard': 0.9144,
            
            // Weight (to grams)
            'g': 1,
            'kg': 1000,
            'lb': 453.592,
            'oz': 28.3495,
            'ton': 1000000,
            
            // Temperature (special handling needed)
            'celsius': 1,
            'fahrenheit': 1,
            'kelvin': 1
        };
    }

    /**
     * Initialize the plugin
     */
    async initialize() {
        console.log(`${this.name} v${this.version} initialized`);
        return {
            status: "success",
            message: "Calculator plugin initialized successfully"
        };
    }

    /**
     * Handle tool calls from the MCP protocol
     */
    async handleToolCall(toolName, parameters) {
        try {
            switch (toolName) {
                case 'calculate':
                    return await this.calculate(parameters.expression);
                case 'convert_units':
                    return await this.convertUnits(parameters.value, parameters.from_unit, parameters.to_unit);
                default:
                    throw new Error(`Unknown tool: ${toolName}`);
            }
        } catch (error) {
            return {
                status: "error",
                error: error.message
            };
        }
    }

    /**
     * Perform mathematical calculation
     */
    async calculate(expression) {
        try {
            // Basic security: only allow numbers, operators, and parentheses
            if (!/^[0-9+\-*/().\s]+$/.test(expression)) {
                throw new Error("Invalid characters in expression");
            }

            // Evaluate the expression safely
            const result = Function(`"use strict"; return (${expression})`)();
            
            if (!isFinite(result)) {
                throw new Error("Result is not a finite number");
            }

            return {
                status: "success",
                result: {
                    expression: expression,
                    value: result,
                    formatted: this.formatNumber(result)
                }
            };
        } catch (error) {
            throw new Error(`Calculation error: ${error.message}`);
        }
    }

    /**
     * Convert between units
     */
    async convertUnits(value, fromUnit, toUnit) {
        try {
            const from = fromUnit.toLowerCase();
            const to = toUnit.toLowerCase();

            // Special handling for temperature
            if (this.isTemperatureUnit(from) || this.isTemperatureUnit(to)) {
                return this.convertTemperature(value, from, to);
            }

            // Check if units are compatible (same category)
            if (!this.areUnitsCompatible(from, to)) {
                throw new Error(`Cannot convert from ${fromUnit} to ${toUnit}: incompatible unit types`);
            }

            // Convert to base unit, then to target unit
            const baseValue = value * this.unitFactors[from];
            const result = baseValue / this.unitFactors[to];

            return {
                status: "success",
                result: {
                    original_value: value,
                    original_unit: fromUnit,
                    converted_value: result,
                    converted_unit: toUnit,
                    formatted: `${this.formatNumber(value)} ${fromUnit} = ${this.formatNumber(result)} ${toUnit}`
                }
            };
        } catch (error) {
            throw new Error(`Unit conversion error: ${error.message}`);
        }
    }

    /**
     * Check if two units are compatible for conversion
     */
    areUnitsCompatible(unit1, unit2) {
        const lengthUnits = ['m', 'km', 'cm', 'mm', 'ft', 'in', 'mile', 'yard'];
        const weightUnits = ['g', 'kg', 'lb', 'oz', 'ton'];
        
        const isLength1 = lengthUnits.includes(unit1);
        const isLength2 = lengthUnits.includes(unit2);
        const isWeight1 = weightUnits.includes(unit1);
        const isWeight2 = weightUnits.includes(unit2);
        
        return (isLength1 && isLength2) || (isWeight1 && isWeight2);
    }

    /**
     * Check if unit is a temperature unit
     */
    isTemperatureUnit(unit) {
        return ['celsius', 'fahrenheit', 'kelvin', 'c', 'f', 'k'].includes(unit.toLowerCase());
    }

    /**
     * Convert temperature units
     */
    convertTemperature(value, from, to) {
        const fromUnit = from.toLowerCase();
        const toUnit = to.toLowerCase();

        // Convert to Celsius first
        let celsius;
        switch (fromUnit) {
            case 'celsius':
            case 'c':
                celsius = value;
                break;
            case 'fahrenheit':
            case 'f':
                celsius = (value - 32) * 5/9;
                break;
            case 'kelvin':
            case 'k':
                celsius = value - 273.15;
                break;
            default:
                throw new Error(`Unknown temperature unit: ${from}`);
        }

        // Convert from Celsius to target unit
        let result;
        switch (toUnit) {
            case 'celsius':
            case 'c':
                result = celsius;
                break;
            case 'fahrenheit':
            case 'f':
                result = celsius * 9/5 + 32;
                break;
            case 'kelvin':
            case 'k':
                result = celsius + 273.15;
                break;
            default:
                throw new Error(`Unknown temperature unit: ${to}`);
        }

        return {
            status: "success",
            result: {
                original_value: value,
                original_unit: from,
                converted_value: result,
                converted_unit: to,
                formatted: `${this.formatNumber(value)}°${from.toUpperCase()} = ${this.formatNumber(result)}°${to.toUpperCase()}`
            }
        };
    }

    /**
     * Format number for display
     */
    formatNumber(num) {
        if (Number.isInteger(num)) {
            return num.toString();
        }
        return parseFloat(num.toFixed(6)).toString();
    }

    /**
     * Get plugin capabilities
     */
    getCapabilities() {
        return {
            tools: ['calculate', 'convert_units'],
            resources: [],
            prompts: []
        };
    }

    /**
     * Cleanup when plugin is unloaded
     */
    async cleanup() {
        console.log(`${this.name} cleanup completed`);
        return {
            status: "success",
            message: "Calculator plugin cleanup completed"
        };
    }
}

// Export the plugin class
if (typeof module !== 'undefined' && module.exports) {
    module.exports = SimpleCalculator;
} else if (typeof window !== 'undefined') {
    window.SimpleCalculator = SimpleCalculator;
}

// Plugin entry point for iTools
async function createPlugin() {
    const calculator = new SimpleCalculator();
    await calculator.initialize();
    return calculator;
}

// Export the entry point
if (typeof module !== 'undefined' && module.exports) {
    module.exports.createPlugin = createPlugin;
}
