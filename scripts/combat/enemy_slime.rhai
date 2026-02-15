// Enemy AI: Slime (Basic enemy)
// Based on PSX entity callback system

// Called every frame to update enemy behavior
fn on_update() {
    // Check if it's our turn
    let hp_percent = get_hp() / get_max_hp();
    
    // Low HP behavior - try to escape or defend
    if hp_percent < 0.3 {
        if random() < 0.5 {
            return "defend";
        }
    }
    
    // Normal behavior - attack
    return "attack";
}

// Called when choosing a target
fn choose_target() {
    let alive_allies = count_alive_allies();
    
    if alive_allies == 1 {
        // Only one target
        return 0;
    }
    
    // Random target selection
    return random_range(0, alive_allies - 1);
}

// Called when taking damage
fn on_damage_taken(damage_amount) {
    // Flash red when damaged
    set_color_target(0x3fc0, 0, 0); // Full red
    set_timer(1, 15); // Flash for 15 frames
}

// Called every frame for visual updates
fn on_animation_update() {
    let timer = get_timer(1);
    
    if timer > 0 {
        // Still flashing red from damage
        // Color interpolation happens automatically
    } else {
        // Return to normal color
        set_color_target(0x3fc0, 0x3fc0, 0x3fc0); // White
    }
}
