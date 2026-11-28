use super::*;

#[test]
fn matriz_r319_e_r320() {
    let casos: &[(&str, &[&str])] = &[
        ("0012345678", &["0012345678"]),
        ("23106.001630/2026-66", &["23106.001630/2026-66"]),
        ("<a>12345678</a>", &[]),
        ("1234<b>5678</b>", &["12345678"]),
        ("1234<br>5678", &[]),
        ("1234&#53;678", &["12345678"]),
        ("<input value=\"12345678\">", &[]),
        ("ABC12345678XYZ", &[]),
        ("á12345678β", &[]),
        ("X23106.001630/2026-66Y", &[]),
        ("<p hidden>12345678</p>", &[]),
        ("<p hidden=\"false\">12345678</p>", &[]),
        ("<p style=\"display:none\">12345678</p>", &["12345678"]),
        ("<select><option>12345678</option></select>", &[]),
        ("<noscript>12345678</noscript>", &[]),
        ("<div hidden><p>12345678</p></div>", &[]),
        ("<p>12345678 e 12345678.</p>", &["12345678", "12345678"]),
        ("0012345678 e 12345678", &["0012345678", "12345678"]),
        ("0016.005764/2024-06; 23106.001630/2026-66", &["0016.005764/2024-06", "23106.001630/2026-66"]),
        ("<p>PJe ids. 56542243 e 56542244.</p><p>guia nº 12345678</p><p>CPF 87654321</p>", &[]),
        ("<strong>ID</strong> 56542243", &[]),
        ("<a>33333333</a> <a>44444444</a> 33333333", &["33333333"]),
        ("<!-- 44444444 --><script>x=55555555;</script><style>x{z:66666666}</style>", &[]),
        ("123.456.789-01 01/02/2026 0020.018553/2026-2 123456789012345678 1234567", &[]),
        ("<p title=\"a > 12345678\">22222222</p>", &["22222222"]),
        ("<p>CPF</p><p>12345678</p>", &["12345678"]),
        ("1<strong>23456789</strong>", &[]),
        ("CPF&nbsp;12345678", &[]),
        ("１２３４５６７８", &[]),
        ("<head><title>12345678</title></head><template>23456789</template><textarea>34567890</textarea><p>45678901</p>", &["45678901"]),
        ("22222222 e 11111111 e 22222222", &["22222222", "22222222", "11111111"]),
        ("12345.123456/2026-123", &[]),
        ("<p>12345678 <a href=\"#\">23456789", &["12345678"]),
        ("<div hidden><div hidden>12345678</div>23456789</div>34567890", &["34567890"]),
        ("<img hidden>12345678", &["12345678"]),
        ("<input>12345678", &["12345678"]),
        ("<button>12345678</button>23456789", &["23456789"]),
        ("<form><input>12345678</form>", &["12345678"]),
        ("1234<span hidden>x</span>5678", &[]),
        ("_12345678_", &[]),
        ("９12345678", &[]),
        ("<p hidden>12345678<p>23456789", &["23456789"]),
        ("<p hidden>12345678</p><p>23456789</p>", &["23456789"]),
        ("<ul><li hidden>12345678<li>23456789</ul>", &["23456789"]),
        ("<ul><li hidden>12345678</li><li>23456789</li></ul>", &["23456789"]),
        ("<table><tr><td hidden>12345678<td>23456789</table>", &["23456789"]),
        ("<table><tr><td hidden>12345678</td><td>23456789</td></tr></table>", &["23456789"]),
        ("á12345678", &[]),
        ("12345678́a", &[]),
    ];
    for (html, esperado) in casos {
        let fonte = HtmlServido::de_teste(html);
        let candidatos = inventariar(&fonte, "", "");
        let textos: Vec<_> = candidatos.iter().flat_map(|c| {
            assert_eq!(c.estado, "candidato");
            std::iter::repeat_n(c.texto.as_str(), c.ocorrencias)
        }).collect();
        assert_eq!(&textos, esperado, "{html}");
        assert_eq!(fonte.as_str(), *html);
    }
}

#[test]
fn exclui_autorreferencia_e_preserva_tipo_e_contagem() {
    let html = HtmlServido::de_teste("11111111 e 23106.001630/2026-66; 22222222");
    let candidatos = inventariar(&html, "11111111", "23106.001630/2026-66");
    assert_eq!(candidatos.len(), 1);
    assert_eq!(candidatos[0].texto, "22222222");
    let html = HtmlServido::de_teste("0012345678 e 23106.001630/2026-66 e 0012345678");
    let candidatos = inventariar(&html, "", "");
    assert_eq!(candidatos.len(), 2);
    assert_eq!(candidatos[0].tipo, Tipo::Documento);
    assert_eq!(candidatos[0].texto, "0012345678");
    assert_eq!(candidatos[0].ocorrencias, 2);
    assert_eq!(candidatos[1].tipo, Tipo::Processo);
    assert_eq!(candidatos[1].texto, "23106.001630/2026-66");
}
