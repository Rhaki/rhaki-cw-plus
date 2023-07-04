use cosmwasm_std::{testing::mock_dependencies, Addr};
use rhaki_cw_plus::auth::{assert_owner, get_owner, set_owner};

#[test]
fn main() {
    let owner = Addr::unchecked("owner_addr");
    let mut deps = mock_dependencies();

    assert_eq!(get_owner(deps.as_mut().storage), None);

    assert_owner(deps.as_mut().storage, &owner).unwrap_err();

    set_owner(deps.as_mut().storage, &owner).unwrap();

    assert_eq!(get_owner(deps.as_mut().storage).unwrap(), owner);

    assert_owner(deps.as_mut().storage, &owner).unwrap();

    assert_owner(deps.as_mut().storage, &Addr::unchecked("rand")).unwrap_err();
}
