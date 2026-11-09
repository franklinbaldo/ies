//! Formulário de prazo baseado em estado SQLite tipado e explicitamente observado.
//! POST sintético modifica somente o SQLite efêmero; nenhuma escrita externa.
use crate::{Estado, Resposta, escapar, parametro_unico};
use iesei::instalacao::Instalacao;
use iesei::prazos::{self, EstadoDoPrazo};
use rusqlite::Connection;

/// Serve o formulário sintético sem inferir "sem prazo" de ausência de captura.
pub(crate) fn responder(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if !matches!(metodo, "GET" | "POST") {
        return Resposta::nova(Estado::MetodoNaoPermitido);
    }
    if parametro_unico(alvo, "acao") != Some("controle_prazo_definir") {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let Some(id) = parametro_unico(alvo, "id_procedimento")
        .filter(|id| id.bytes().all(|byte| byte.is_ascii_digit()))
    else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    let Ok(Some(estado)) = prazos::ler(banco, instalacao, cenario, id) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if metodo == "POST" {
        return gravar_prazo(corpo, banco, instalacao, cenario, id);
    }
    let (radio_data, radio_dias, radio_concluido, data, dias) = match estado {
        EstadoDoPrazo::SemPrazo => ("", "", "", String::new(), String::new()),
        EstadoDoPrazo::Data(data) => (" checked", "", "", escapar(&data), String::new()),
        EstadoDoPrazo::Dias(dias) => ("", " checked", "", String::new(), dias.to_string()),
        EstadoDoPrazo::Concluido => ("", "", " checked", String::new(), String::new()),
    };
    let id = escapar(id);
    let assinatura = crate::protocolo_sintetico::INFRA_HASH;
    Resposta::html(format!(
        "<html><body><form id='frmControlePrazoCadastro' method='post' \
         action='controlador.php?acao=controle_prazo_definir&amp;id_procedimento={id}&amp;infra_hash={assinatura}'>\
         <input type='hidden' name='hdnInfraTipoPagina' value='2'>\
         <input type='hidden' name='hdnIdControlePrazo' value=''>\
         <input type='hidden' name='hdnIdProcedimentos' value='{id}'>\
         <input type='radio' name='rdoPrazo' value='1'{radio_data}>\
         <input type='text' name='txtPrazo' value='{data}'>\
         <input type='radio' name='rdoPrazo' value='2'{radio_dias}>\
         <input type='text' name='txtDias' value='{dias}'>\
         <input type='radio' name='rdoPrazo' value='3'{radio_concluido}>\
         <input type='checkbox' name='chkSinDiasUteis' value='on'>\
         <button type='submit' name='sbmDefinirControlePrazo'>Salvar</button>\
         </form></body></html>"
    ))
}

/// A escrita é uma hipótese sintética, não uma reprodução do POST do SEI.
/// O formulário deve pertencer ao processo e conter um pedido único.
fn gravar_prazo(
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
    id: &str,
) -> Resposta {
    let pedido = (
        campo_unico(corpo, "hdnIdProcedimentos"),
        campo_unico(corpo, "rdoPrazo"),
        campo_unico(corpo, "txtPrazo"),
        campo_unico(corpo, "txtDias"),
    );
    let (Some(processo), Some(opcao), Some(data), Some(dias)) = pedido else {
        return Resposta::nova(Estado::RequisicaoInvalida);
    };
    if processo != id {
        return Resposta::nova(Estado::RequisicaoInvalida);
    }
    let pedido = match (opcao.as_str(), data.is_empty(), dias.is_empty()) {
        ("1", false, true) => EstadoDoPrazo::Data(data),
        ("2", true, false) => {
            let Some(dias) = dias.parse::<u16>().ok().filter(|n| (1..=999).contains(n)) else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            EstadoDoPrazo::Dias(dias)
        }
        _ => return Resposta::nova(Estado::RequisicaoInvalida),
    };
    match prazos::guardar(banco, instalacao, cenario, id, &pedido) {
        Ok(()) => Resposta::redirecionar(format!(
            "controlador.php?acao=controle_prazo_definir&resultado=1&id_procedimento={id}"
        )),
        Err(_) => Resposta::nova(Estado::DadosInsuficientes),
    }
}

fn campo_unico(corpo: &str, campo: &str) -> Option<String> {
    let mut valores = corpo
        .split('&')
        .filter_map(|parte| parte.split_once('='))
        .filter(|(nome, _)| *nome == campo)
        .map(|(_, valor)| valor);
    let valor = valores.next()?;
    if valores.next().is_some() {
        return None;
    }
    let mut bytes = Vec::new();
    let mut entrada = valor.as_bytes().iter().copied();
    while let Some(byte) = entrada.next() {
        match byte {
            b'+' => bytes.push(b' '),
            b'%' => {
                let alto = char::from(entrada.next()?).to_digit(16)?;
                let baixo = char::from(entrada.next()?).to_digit(16)?;
                bytes.push(u8::try_from(alto * 16 + baixo).ok()?);
            }
            outro => bytes.push(outro),
        }
    }
    String::from_utf8(bytes).ok()
}


fn unidade_da_lista(
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Option<String> {
    iesei::banco_cassettes::ler(banco, instalacao, cenario)
        .ok()
        .flatten()
        .map(|controle| controle.unidade.as_str().to_owned())
}

/// O menu da unidade oferece a lista somente quando ha prazo em vigor.
pub(crate) fn acrescentar_menu(
    mut resposta: Resposta,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    if resposta.estado != Estado::Ok {
        return resposta;
    }
    let Some(unidade) = unidade_da_lista(banco, instalacao, cenario) else {
        return resposta;
    };
    let Ok(linhas) = prazos::listar(banco, instalacao, cenario, &unidade) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    if !linhas.is_empty() && resposta.corpo.contains("</body>") {
        let assinatura = crate::protocolo_sintetico::INFRA_HASH;
        let link = format!(
            "<a href='controlador.php?acao=controle_prazo_listar&amp;infra_hash={assinatura}'>Controle de Prazos</a>"
        );
        resposta.corpo = resposta.corpo.replacen("</body>", &format!("{link}</body>"), 1);
    }
    resposta
}

/// Lista de prazos da unidade. A identidade do item de exclusao usa, por
/// hipotese sintetica, o id do processo; o SEI pode usar id distinto.
pub(crate) fn responder_lista(
    metodo: &str,
    alvo: &str,
    corpo: &str,
    banco: &Connection,
    instalacao: &Instalacao,
    cenario: &str,
) -> Resposta {
    let Some(unidade) = unidade_da_lista(banco, instalacao, cenario) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    let Ok(linhas) = prazos::listar(banco, instalacao, cenario, &unidade) else {
        return Resposta::nova(Estado::DadosInsuficientes);
    };
    match (parametro_unico(alvo, "acao"), metodo) {
        (Some("controle_prazo_listar"), "GET") => {
            let hash = crate::protocolo_sintetico::INFRA_HASH;
            let mut pagina = format!(
                "<html><body><form id='frmControlePrazoLista' method='post'                  action='controlador.php?acao=controle_prazo_listar&amp;infra_hash={hash}'>                 <input type='hidden' name='hdnInfraItemId' value=''>                 <table><thead><tr><th>Processo</th><th>Prazo</th><th>Acoes</th></tr></thead><tbody>"
            );
            for linha in linhas {
                let id = escapar(&linha.id_procedimento);
                let nup = escapar(&linha.nup);
                let prazo = match linha.estado {
                    EstadoDoPrazo::Data(data) => escapar(&data),
                    EstadoDoPrazo::Dias(dias) => dias.to_string(),
                    EstadoDoPrazo::SemPrazo | EstadoDoPrazo::Concluido => continue,
                };
                pagina.push_str(&format!(
                    "<tr><td>{nup}</td><td>{prazo}</td><td><a href='#'                      onclick=\"acaoExcluir('{id}','{nup}');\">Excluir</a></td></tr>"
                ));
            }
            pagina.push_str(&format!(
                "</tbody></table></form><script>function acaoExcluir(id, nup) {{                  document.getElementById('frmControlePrazoLista').action =                  'controlador.php?acao=controle_prazo_excluir&infra_hash={hash}';                  }}</script></body></html>"
            ));
            Resposta::html(pagina)
        }
        (Some("controle_prazo_excluir"), "POST") => {
            let Some(id) = campo_unico(corpo, "hdnInfraItemId")
                .filter(|id| id.bytes().all(|c| c.is_ascii_digit()))
            else {
                return Resposta::nova(Estado::RequisicaoInvalida);
            };
            if !linhas.iter().any(|linha| linha.id_procedimento == id) {
                return Resposta::nova(Estado::RequisicaoInvalida);
            }
            match prazos::excluir(banco, instalacao, cenario, &id) {
                Ok(true) => Resposta::redirecionar(
                    "controlador.php?acao=controle_prazo_listar&acao_origem=controle_prazo_excluir"
                        .to_owned(),
                ),
                Ok(false) => Resposta::nova(Estado::RequisicaoInvalida),
                Err(_) => Resposta::nova(Estado::DadosInsuficientes),
            }
        }
        (Some("controle_prazo_listar" | "controle_prazo_excluir"), _) => {
            Resposta::nova(Estado::MetodoNaoPermitido)
        }
        _ => Resposta::nova(Estado::RequisicaoInvalida),
    }
}
