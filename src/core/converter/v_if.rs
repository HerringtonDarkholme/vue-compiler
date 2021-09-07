#[cfg(test)]
mod test {
    fn test() {
        let cases = vec![
            r#"
<p v-if="false">a</p>
<p v-else v-if="true">b</p>
<p v-else>c</p>"#,
        ];
    }
}
