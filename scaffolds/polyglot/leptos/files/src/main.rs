use leptos::*;
use leptos_router::*;

#[component]
fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes>
                <Route path="/" view=Home/>
            </Routes>
        </Router>
    }
}

#[component]
fn Home() -> impl IntoView {
    let (count, set_count) = create_signal(0);

    view! {
        <div class="home">
            <h1>"{{ name }}"</h1>
            <p>"{{ description }}"</p>
            <div class="card">
                <button on:click=move |_| set_count.update(|n| *n += 1)>
                    "count is " {count}
                </button>
            </div>
        </div>
    }
}

fn main() {
    console_log::init_with_level(log::Level::Debug).expect("error initializing log");
    mount_to_body(|| view! { <App/> })
}
