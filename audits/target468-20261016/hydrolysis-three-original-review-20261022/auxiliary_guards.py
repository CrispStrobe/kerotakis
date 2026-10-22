"""Prospective narrow admission for source-beff Vessel pools; no chemistry oracle."""
from decimal import Decimal as D

def finite_scalar(value):
    assert type(value)in(D,int) and (type(value)is int or value.is_finite()), 'finite typed JSON scalar required'
    return value

def check_owned_pools(v):
    # Source beff Vessel serde defaults: vectors may be omitted, options may be null.
    vectors=('unresolved_materials','material_objects','surfaces','electrodes','exchanges','adsorbed','solid_solutions','surface_colours','charred_materials')
    options=('soap_scum','lemon_paper_mark','surface_particles','emulsion')
    for key in vectors:
        if key in v:assert type(v[key])is list and v[key]==[], ('unbudgeted pool',key)
    for key in options:assert v.get(key)is None, ('unbudgeted pool',key)
    assert type(v['nuclides'])is dict and set(v['nuclides'])=={'inventory'} and v['nuclides']['inventory']=={} and type(v['nuclides']['inventory'])is dict, 'unbudgeted nuclide inventory'
    foam=v['foam'];assert type(foam)is dict and set(foam)=={'trapped_gas_liters','volume_liters','peak_volume_liters'}
    for value in foam.values():assert finite_scalar(value)==0, 'unbudgeted foam history/gas'
    assert v['headspace']=={'boundary':'open'}, 'unbudgeted finite gas headspace'
    assert finite_scalar(v['pending_co2_transfer_mol'])==0, 'unbudgeted pending CO2'
    for key in ['elapsed_seconds','temperature','pressure','excess_enthalpy_j','solute_charge','free_hydroxide','free_proton']:finite_scalar(v[key])
    assert v['temperature']>0 and v['pressure']>0
    if v['co2_partial_pressure_atm']is not None:assert finite_scalar(v['co2_partial_pressure_atm'])>=0
    # All actual source-beff serializable primary fields admitted explicitly.
    # lots/solution/resolved/step_start are provenance or derived state, not extra matter.
    allowed=set(vectors+options)|{'elapsed_seconds','nuclides','excess_enthalpy_j','id','label','contents','temperature','pressure','thermal_mode','headspace','foam','solute_charge','step_start','free_hydroxide','free_proton','co2_partial_pressure_atm','pending_co2_transfer_mol','solution','lots','resolved'}
    assert set(v)<=allowed, ('unknown Vessel field: HOLD',set(v)-allowed)
