//! Fluxos inteiros com transporte roteirizado, sem rede ou dados reais.
//! Oráculo: Todos src/todos/documentos.py @323eb01; forma Web 4.0.
//! O orçamento inclui abertura, escolha, cadastro, escrita e verificação.
use super::*;
use crate::arvore_processo::tests::no_documento;
use crate::processo_concluir::tests_apoio as a;
use crate::propriedades_processo::tests_apoio::acao_da_barra;
use crate::tela_html::HtmlServido;

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
) -> (Result<ResultadoCriacao, ErroDocumentoCriar>, Vec<a::Pedido>) {
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
        Err(erro) => panic!("prévia falhou: {erro:?}"),
    };
    assert!(matches!(previa.estado, EstadoCriacao::Previa));
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos, pedidos_antes_da_escrita());
}

#[test]
fn dado_descricao_e_nivel_quando_vejo_previa_entao_sao_exibidos_sem_post() {
    // Dado um pedido com descricao e nivel explicitos;
    // Quando executo a previa sem confirmar;
    // Entao ambos os dados estao disponiveis para apresentacao na CLI.
    let (resultado, pedidos) = executar(false, vec![]);
    let previa = match resultado {
        Ok(previa) => previa,
        Err(erro) => panic!("previa falhou: {erro:?}"),
    };
    assert_eq!(previa.descricao.as_deref(), Some("Ensaio"));
    assert_eq!(previa.nivel.as_deref(), Some("publico"));
    assert_eq!(previa.requisicoes, 5);
    assert_eq!(pedidos, pedidos_antes_da_escrita());
}

#[test]
fn confirmar_unico_post_releitura_e_numero_sei_com_sete_requisicoes() {
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::r302("controlador.php?acao=documento_visualizar&id_documento=22"),
            a::ok(arvore_verificadora(1)),
        ],
    );
    let confirmacao = match resultado {
        Ok(confirmacao) => confirmacao,
        Err(erro) => panic!("confirmação sintética recusada: {erro:?}"),
    };
    assert!(matches!(
        &confirmacao.estado,
        EstadoCriacao::Criado { numero_sei } if numero_sei == "0000102"
    ));
    assert_eq!(confirmacao.requisicoes, 7);
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn dado_aceite_arvore_visualizar_quando_confirmo_entao_retorna_numero_sei() {
    // Dado o 302 de cadastro para arvore_visualizar observado no oraculo
    // Todos tests/test_escritas_diretas_010.py @323eb01;
    // Quando executo ies documento criar --confirmar com --nivel publico;
    // Entao um POST e a releitura independente comprovam o numero SEI.
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::r302("controlador.php?acao=arvore_visualizar&id_documento=22"),
            a::ok(arvore_verificadora(1)),
        ],
    );
    let criado = match resultado {
        Ok(criado) => criado,
        Err(erro) => panic!("aceite recusado: {erro:?}"),
    };
    assert!(matches!(
        &criado.estado,
        EstadoCriacao::Criado { numero_sei } if numero_sei == "0000102"
    ));
    assert_eq!(criado.requisicoes, 7);
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(
        pedidos, esperados,
        "5 GET de previa, POST unico, GET de conferencia"
    );
}

#[test]
fn dado_redirect_arvore_com_id_alheio_quando_confirmo_entao_nao_confirmado() {
    // Dado um 302 arvore_visualizar que aponta para outro id de documento;
    // Quando executo a confirmacao e a releitura mostra um novo documento;
    // Entao nao atribuo causalidade ao POST nem repito a escrita.
    let (resultado, pedidos) = executar(
        true,
        vec![
            a::r302("controlador.php?acao=arvore_visualizar&id_documento=999"),
            a::ok(arvore_verificadora(1)),
        ],
    );
    assert!(matches!(
        resultado,
        Err(ErroDocumentoCriar::Confirmacao(
            ErroConfirmacaoDaCriacao::NaoConfirmado
        ))
    ));
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn dois_documentos_apos_post_sao_incerteza_sem_retry() {
    let (_resultado, pedidos) = executar(
        true,
        vec![
            a::r302("controlador.php?acao=documento_visualizar&id_documento=22"),
            a::ok(arvore_verificadora(2)),
        ],
    );
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

fn executar_com_cadastro_sintetico(
    html_cadastro: String,
) -> (Result<ResultadoCriacao, ErroDocumentoCriar>, Vec<a::Pedido>) {
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
fn botao_salvar_observado_publica_controle_de_gravacao() {
    // A forma servida inclui o botão habilitado e o formulário de cadastro.
    assert!(cadastro_oferece_salvar(&HtmlServido::de_teste(&cadastro())));
}

#[test]
fn botao_salvar_sem_name_conserva_post_unico_e_releitura() {
    // Um button type=button com onclick nao envia seu atributo name.
    // A ausencia dele nao remove a oferta de salvar publicada na tela.
    let html = cadastro().replace(" name=\"btnSalvar\"", "");
    assert!(cadastro_oferece_salvar(&HtmlServido::de_teste(&html)));
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(html));
    respostas.push(a::r302(
        "controlador.php?acao=documento_visualizar&id_documento=22",
    ));
    respostas.push(a::ok(arvore_verificadora(1)));
    let (resultado, pedidos) = a::rodar_com(&amb, respostas, |sessao, nup, indice, rede| {
        let pedido = PedidoCriacao {
            nup,
            unidade: None,
            tipo: "Despacho",
            descricao: Some("Ensaio"),
            nivel: Some("publico"),
        };
        executar_criacao(sessao, &pedido, indice, rede, true)
    });
    assert!(matches!(
        resultado,
        Ok(ResultadoCriacao {
            estado: EstadoCriacao::Criado { numero_sei },
            requisicoes: 7,
            ..
        }) if numero_sei == "0000102"
    ));
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
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
        let (_resultado, pedidos) = executar_com_cadastro_sintetico(html);
        assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST criador");
    }
}

#[test]
fn dois_botoes_salvar_identicos_sao_permitidos_na_captura() {
    // Topo e rodape da captura 4.0.9 repetem intencionalmente btnSalvar.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(cadastro_oferece_salvar(&HtmlServido::de_teste(
        &original.replace("</body>", &format!("{botao}</body>"))
    )));
}

#[test]
fn controle_salvar_dentro_de_template_nao_autoriza_post_criador() {
    // O conteúdo de template fica fora da árvore, como no navegador: sem
    // btnSalvar na tela, a escrita não é comprovada.
    let original = cadastro();
    let botao = "<button type=\"button\" id=\"btnSalvar\" name=\"btnSalvar\"\n          onclick=\"confirmarDados()\">Salvar</button>";
    assert!(original.contains(botao));
    let html = original.replace(botao, &format!("<template>{botao}</template>"));
    let (resultado, pedidos) = executar_com_cadastro_sintetico(html);
    assert!(matches!(
        resultado,
        Err(ErroDocumentoCriar::SubmissaoNaoComprovada)
    ));
    assert_eq!(pedidos, pedidos_antes_da_escrita(), "5 GET, 0 POST");
}
#[test]
fn incerteza_conserva_contagem_da_rede_externa() {
    // Mesmo com efeito incerto, cinco GET de prévia, um POST e um GET
    // de verificação são observáveis na fronteira; sem retry.
    let amb = a::ambiente();
    let mut respostas = a::abertura(arvore_inicial());
    respostas.push(a::ok(lista_de_tipos()));
    respostas.push(a::ok(cadastro()));
    respostas.push(a::ok(
        "<html>resposta genérica sem aceite</html>".to_owned(),
    ));
    respostas.push(a::ok(arvore_verificadora(1)));
    let ((resultado, requisicoes), pedidos) =
        a::rodar_com(&amb, respostas, |sessao, nup, indice, envio| {
            let pedido = PedidoCriacao {
                nup,
                unidade: None,
                tipo: "Despacho",
                descricao: Some("Ensaio"),
                nivel: Some("publico"),
            };
            let mut contagem = Contador::novo(envio);
            let resultado = executar_criacao(sessao, &pedido, indice, &mut contagem, true);
            (resultado, contagem.feitas())
        });
    assert!(matches!(
        resultado,
        Err(ErroDocumentoCriar::Confirmacao(
            ErroConfirmacaoDaCriacao::NaoConfirmado
        ))
    ));
    assert_eq!(
        requisicoes, 7,
        "inclui releitura mesmo após resposta inconclusiva"
    );
    let mut esperados = pedidos_antes_da_escrita();
    esperados.push(pedido_criador());
    esperados.push(a::get(&a::url_arvore()));
    assert_eq!(pedidos, esperados);
}

#[test]
fn controle_salvar_e_formulario_servidos_nao_dependem_de_interpretar_js() {
    // Ablação #246 (🦡): a autorização deriva de botão e formulário
    // observados; a lógica interna do JS não é o contrato HTTP do IES.
    // O servidor ainda decide se aceita o POST e a árvore comprova o efeito.
    let html = HtmlServido::de_teste(&cadastro_com_script(
        "/* lógica do navegador não interpretada */",
    ));
    assert!(cadastro_oferece_salvar(&html));
}
