// Enemy AI: Boss (Advanced enemy with multiple phases)
// Demonstrates complex AI based on PSX callback patterns

// Global state (persisted between calls via static variables in script)
let phase = 1;
let turn_count = 0;
let used_special = false;

// Called every frame to update boss behavior
fn on_update() {
    turn_count += 1;
    
    let hp_percent = get_hp() / get_max_hp();
    let alive_enemies = count_alive_enemies();
    
    // Phase transitions based on HP
    if hp_percent < 0.25 && phase < 3 {
        phase = 3;
        set_color_target(0x3fc0, 0, 0x3fc0); // Purple (enraged)
        return "special_ultimate";
    } else if hp_percent < 0.5 && phase < 2 {
        phase = 2;
        set_color_target(0x3fc0, 0x2000, 0); // Orange (phase 2)
        return "special_heal";
    }
    
    // Phase 3 - Aggressive (below 25% HP)
    if phase == 3 {
        if turn_count % 3 == 0 {
            return "special_multi_attack";
        }
        return "attack_strong";
    }
    
    // Phase 2 - Balanced (25-50% HP)
    if phase == 2 {
        if alive_enemies <= 1 && random() < 0.3 {
            return "special_summon";
        }
        
        if turn_count % 4 == 0 {
            return "special_attack";
        }
        
        return random() < 0.6 ? "attack" : "defend";
    }
    
    // Phase 1 - Normal (above 50% HP)
    if turn_count % 5 == 0 && !used_special {
        used_special = true;
        return "special_attack";
    }
    
    return "attack";
}

// Choose target with priority logic
fn choose_target() {
    let alive_allies = count_alive_allies();
    
    if phase == 3 {
        // In phase 3, target lowest HP ally
        return find_weakest_target();
    }
    
    // Otherwise random
    return random_range(0, alive_allies - 1);
}

// Helper function to find weakest target
fn find_weakest_target() {
    // TODO: This would need engine support to query all targets
    // For now, just random
    return random_range(0, count_alive_allies() - 1);
}

// Visual feedback when taking damage
fn on_damage_taken(damage_amount) {
    // More dramatic flash in later phases
    if phase == 3 {
        set_color_target(0x3fc0, 0, 0); // Bright red
        set_timer(1, 30); // Longer flash
    } else {
        set_color_target(0x3000, 0, 0); // Normal red
        set_timer(1, 15);
    }
}

// Animation updates
fn on_animation_update() {
    let timer = get_timer(1);
    
    if timer > 0 {
        // Flashing from damage
    } else {
        // Restore phase color
        if phase == 3 {
            set_color_target(0x3fc0, 0, 0x3fc0); // Purple
        } else if phase == 2 {
            set_color_target(0x3fc0, 0x2000, 0); // Orange
        } else {
            set_color_target(0x3fc0, 0x3fc0, 0x3fc0); // White
        }
    }
}
