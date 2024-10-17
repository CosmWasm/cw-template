use cw_orch::{anyhow, prelude::*};
use {{project-name | snake_case}}::{% raw %}{{% endraw %}
    interface::{{project-name | upper_camel_case}}I,
    msg::{ExecuteMsgFns, InstantiateMsg, QueryMsgFns},
};

pub fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok(); // Used to load the `.env` file if any
    env_logger::init(); // Used to log contract and chain interactions

    let network = networks::PION_1;
    let chain = DaemonBuilder::new(network.clone()).build()?;

    let counter = {{project-name | upper_camel_case}}I::new(chain);

    counter.upload()?;

    let msg = InstantiateMsg {% raw %}{{% endraw %}{% unless minimal %} count: 1i32 {% endunless %}};
    counter.instantiate(&msg, None, &[])?;{% unless minimal %}

    counter.increment()?;
    let count = counter.get_count()?;
    assert_eq!(count.count, 1);{% endunless %}
    Ok(())
}
