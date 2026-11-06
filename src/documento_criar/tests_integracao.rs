//! Fluxos inteiros com transporte roteirizado, sem rede ou dados reais.
//! Oráculo: Todos src/todos/documentos.py @323eb01; forma Web 4.0.
//! O orçamento inclui abertura, escolha, cadastro, escrita e verificação.
use super::*;
use crate::arvore_processo::tests::no_documento;
use crate::processo_concluir::tests_apoio as a;
use crate::propriedades_processo::tests_apoio::acao_da_barra;

fn link_escolha() -> String {
    format!(
        "controlador.php?acao=documento_escolher_tipo&id_procedimento=10&infra_hash={}",
        "e".repeat(64)
    )
}

fn link_cadastro() -> String {
    format!(
        "controlador.php?acao=documento_gerar&id_procedimento=10&infra_hash={}",
        "f".repeat(64)
    )
}

fn url(relativo: &str) -> String {
    format!("{}/{}", a::ORIGEM, relativo)
}

fn arvore_inicial() -> String {
    a::arvore(&[a::UNIDADE], &[acao_da_barra(&link_escolha())])
}

fn lista_de_tipos() -> String {
    let href = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><table id="tblSeries"><tr><td><a href="{href}">Despacho</a></td></tr></table></html>"#
    )
}

fn cadastro_com_script(script: &str) -> String {
    let action = link_cadastro().replace('&', "&amp;");
    format!(
        r#"<html><head><script>{script}</script></head><body>
        <label id="lblSerieTitulo">Despacho</label>
        <form id="frmDocumentoCadastro" method="post" action="{action}">
          <input type="hidden" name="hdnIdSerie" value="291">
          <input type="hidden" name="hdnFlagDocumentoCadastro" value="1">
          <input type="radio" id="optPublico" name="rdoNivelAcesso" value="0">
          <input type="text" id="txtDescricao" name="txtDescricao" value="">
        </form>
        <button type="button" id="btnSalvar" name="btnSalvar"
          onclick="confirmarDados()">Salvar</button></body></html>"#
    )
}

fn cadastro() -> String {
    // Forma de submeter() servida no SEI 4.0.9 (Todos @323eb01).
    cadastro_com_script(
        r"function confirmarDados() {
  if (OnSubmitForm()) {
    submeter();
  }
}
function submeter() {
  desabilitarBotaoSalvar(true);
  document.getElementById('hdnFlagDocumentoCadastro').value = '2';
  document.getElementById('frmDocumentoCadastro').submit();
}",
    )
}

fn arvore_verificadora(novos: usize) -> String {
    let mut nos = vec![
        crate::arvore_processo::tests::no_processo(0, a::ID, a::NUP),
        no_documento(1, "21", a::ID, "0000101"),
    ];
    if novos >= 1 {
        nos.push(no_documento(2, "22", a::ID, "0000102"));
    }
    if novos >= 2 {
        nos.push(no_documento(3, "23", a::ID, "0000103"));
    }
    crate::arvore_processo::tests::arvore(&nos)
}

fn executar(
    confirmar: bool,
    extra: Vec<a::Resposta>,
) -> (Result<PreviaCriacao, ErroPreviaCriacao>, Vec<a::Pedido>) {
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(cadastro()));
    respostas.extend(extra);
    a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: Some("Ensaio"),
            nivel: Some("publico"),
        };
        executar_criacao(sessao, &pedido, indice, rede, confirmar)
    })
}

fn pedidos_antes_da_escrita() -> Vec<a::Pedido> {
    let mut pedidos = a::pedidos_da_abertura();
    pedidos.push(a::get(&url(&link_escolha())));
    pedidos.push(a::get(&url(&link_cadastro())));
    pedidos
}

fn pedido_criador() -> a::Pedido {
    a::post(
        &url(&link_cadastro()),
        "hdnIdSerie=291&hdnFlagDocumentoCadastro=2&rdoNivelAcesso=0&txtDescricao=Ensaio",
    )
}

#[test]
fn previa_completa_so_cinco_gets_e_nenhum_post() {
    let (resultado, pedidos) = executar(false, vec![]);
    let previa = match resultado {
        Ok(previa) => previa,
        Err(erro) => panic!("prévia falhou: {}", erro.subcodigo()),
    };
    assert!(!previa.confirmado);
    assert_eq!(previa.numero_sei, None);
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos, pedidos_antes_da_escrita());
}

#[test]
fn confirmar_unico_post_releitura_e_numero_sei_com_sete_requisicoes() {
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::ok("<html>salvo</html>".to_owned()),
            a::ok(arvore_verificadora(1)),
        ],
    );
    let confirmacao = match resultado {
        Ok(confirmacao) => confirmacao,
        Err(erro) => panic!("confirmação sintética recusada: {}", erro.subcodigo()),
    };
    assert!(confirmacao.confirmado);
    assert_eq!(confirmacao.numero_sei.as_deref(), Some("0000102"));
    assert_eq!(confirmacao.requisicoes, 7);
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn dois_documentos_apos_post_sao_incerteza_sem_retry() {
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::ok("<html>salvo</html>".to_owned()),
            a::ok(arvore_verificadora(2)),
        ],
    );
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("efeito_incerto_na_criacao_de_documento")
    );
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

fn executar_com_cadastro_sintetico(
    html_cadastro: String,
) -> (Result<PreviaCriacao, ErroPreviaCriacao>, Vec<a::Pedido>) {
    let ambiente = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(html_cadastro));
    a::rodar_com(&ambiente, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: Some("Ensaio"),
            nivel: Some("publico"),
        };
        executar_criacao(sessao, &pedido, indice, rede, true)
    })
}

#[test]
fn funcao_submeter_observada_no_sei_409_publica_sinalizador() {
    // O controle positivo usa a forma completa observada, nunca uma
    // atribuicao sintetica isolada.
    assert!(cadastro_publica_script_de_criacao(&cadastro()));
}

#[test]
fn atribuicao_isolada_nao_comprova_caminho_de_submissao_e_recusa_post() {
    // A forma observada em Todos documento_gerar.html @323eb01
    // exige a funcao submeter(), a desabilitacao e o submit do formulario.
    // Uma atribuicao isolada pode existir numa tela que nunca envia.
    // Nao executar JS; recusar ANTES da escrita, com cinco GET exatos.
    let (resultado, pedidos) = executar_com_cadastro_sintetico(cadastro_com_script(
        "document.getElementById('hdnFlagDocumentoCadastro').value = '2';",
    ));
    assert_eq!(
        resultado.err().map(|erro| erro.subcodigo()),
        Some("submissao_do_documento_nao_observada"),
    );
    assert_eq!(
        pedidos,
        pedidos_antes_da_escrita(),
        "cinco GET de previa, nenhum POST criador",
    );
}

#[test]
fn script_inerte_ou_duplicado_nao_autoriza_post_criador() {
    // O HTML de treinamento não prova uma instrução executável quando a
    // sequência aparece em template, JSON, comentário ou script duplicado.
    // O teste não executa JavaScript nem faz requisições ao SEI.
    let casos = [
        cadastro()
            .replace("<script>", "<template>")
            .replace("</script>", "</template>"),
        cadastro().replace("<script>", "<script type=\"application/json\">"),
        cadastro()
            .replace("<script>", "<script>/*")
            .replace("</script>", "*/</script>"),
        cadastro().replace(
            "</script>",
            "</script><script>document.getElementById('hdnFlagDocumentoCadastro').value = '2';</script>",
        ),
        // Script remoto ignora o corpo inline: não é prova de envio.
        cadastro().replace("<script>", "<script src=\"/static/externo.js\">"),
        // A linha aparece apenas como texto de um template literal JS
        // multilinha, sem instrução executada pelo navegador.
        cadastro().replace(
            "<script>",
            "<script>const amostra = `\n",
        ).replace(
            "</script>",
            "\n`;\n</script>",
        ),
        // Atribuição em ramo comprovadamente inatingível: nenhuma escrita.
        cadastro()
            .replace("<script>", "<script>if (false) {\n")
            .replace("</script>", "\n}</script>"),
        // Uma função arbitrária que não integra o caminho de salvar não
        // comprova a operação, ainda que seu corpo contenha a instrução.
        cadastro()
            .replace("<script>", "<script>function exemploNaoChamado() {\n")
            .replace("</script>", "\n}</script>"),
        // Mesmo uma função chamada submeter não autoriza instrução
        // dentro de seu próprio ramo não alcançável.
        cadastro()
            .replace("<script>", "<script>function submeter() {\nif (false) {\n")
            .replace("</script>", "\n}\n}</script>"),
    ];
    for html in casos {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada")
        );
        assert_eq!(
            pedidos,
            pedidos_antes_da_escrita(),
            "somente os cinco GETs de prévia; nenhum POST criador"
        );
    }
}

#[test]
fn salvar_sem_botao_observado_ou_com_acionador_divergente_recusa_antes_do_post() {
    // Captura Todos documento_gerar.html @323eb01: linhas 810 e 1061
    // publicam dois botoes btnSalvar habilitados e onclick=confirmarDados().
    // Nao basta uma funcao submeter() morta no script para autorizar escrita.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(original.contains(botao));
    let casos = [
        original.replace(botao, ""),
        original.replace("onclick=\"confirmarDados()\"", "onclick=\"outraAcao()\""),
        original.replace("type=\"button\"", "type=\"submit\""),
        original.replace("onclick=\"confirmarDados()\"", "disabled onclick=\"confirmarDados()\""),
        // Um segundo botao divergente nao pode ser ignorado so porque
        // o primeiro continua valido.
        original.replace(
            "</body>",
            "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\" onclick=\"outraAcao()\">Salvar</button></body>",
        ),
    ];
    for html in casos {
        let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(
            resultado.err().map(|erro| erro.subcodigo()),
            Some("submissao_do_documento_nao_observada")
        );
        assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST criador");
    }
}

#[test]
fn dois_botoes_salvar_identicos_sao_permitidos_na_captura() {
    // Topo e rodape da captura 4.0.9 repetem intencionalmente btnSalvar.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(cadastro_publica_script_de_criacao(
        &original.replace("</body>", &format!("{botao}</body>"))
    ));
}
