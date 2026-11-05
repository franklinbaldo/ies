---
type: Guide
title: Erros tipados no IES
description: Condições específicas, códigos estáveis e tratamento seguro no Rust
---

# Erros tipados

O IES adota a disciplina do [Todos](https://github.com/franklinbaldo/todos/blob/main/docs/agentes/erros.md):
cada condição semanticamente distinta tem identidade tipada; mensagem
não determina lógica de tratamento. Em Rust, variantes de `enum` substituem
a hierarquia de subclasses Python quando a família é fechada.

## Regras

- Declare erro junto à operação ou adaptador que o produz; não faça
  de falhas técnicas entidades administrativas do SEI.
- Use variante específica por condição, com campos tipados quando
  contexto for necessário. Não construa uma variante genérica com
  mensagem variável para simular casos diferentes.
- `subcodigo` identifica a condição de modo estável; a mensagem
  apresentada na CLI pode evoluir sem quebrar automações.
- Preserve o erro concreto até a borda de apresentação. Não inspecione
  o texto para tomar decisões; evite `String` como tipo de erro
  para condições conhecidas.
- A categoria pública descreve o próximo passo do chamador.
  Não criar um grande `ErroIES` para todos os módulos.
- Escritas com resultado incerto nunca recebem retry ou troca de
  transporte automática: verificar por leitura antes de repetir.
- Bugs de programação não devem ser convertidos silenciosamente em
  erros esperados de negócio.

## Primeiro recorte aplicado

`src/historico.rs` devolve `ErroHistorico` com variantes como
`NumeroDoHistoricoDiferenteDoSolicitado`, `HistoricoAusente` e
`LinhaInvalida`. As funções não devolvem mais códigos como
`&'static str`; o texto público deriva de `subcodigo()` na CLI.
Testes comparam variantes, não mensagens. Nomes descrevem a condição\nobservada, evitando qualificadores vagos como `ambíguo` e `divergente`.\nO subcódigo `numero_do_historico_diferente_do_solicitado` identifica\na condição observada, sem compromisso artificial com clientes inexistentes.

`ErroCaixa` e `FalhaTransporte` já utilizam variantes específicas.
Revisões posteriores devem acrescentar dados úteis e padronizar
categoria e apresentação, sem alterar códigos públicos desnecessariamente.

## Dados de apresentação nas recusas HTTP

`FalhaTransporte` carrega `StatusHttpRecusado` (status e mensagem
saneada do SEI) em 3xx, 401/403, 429, 5xx e outro status; por isso
deixou de ser `Copy`. A variante e o subcódigo continuam sendo a
classificação: o status e a mensagem são só apresentação e nenhum
código os inspeciona para decidir tratamento. Ver o
[contrato da CLI](../contracts/cli.md).

## Bloqueio de segurança da instalação

`FalhaTransporte::BloqueioDeSegurancaDaInstalacao`
(`bloqueio_de_seguranca_da_instalacao`) é o 403 com a página de bloqueio
do WAF de GO (03/11/2026, depois de rajadas): `<title>Acesso Negado</title>`
e a frase "bloqueada por nossa política de segurança" no texto visível
do corpo; ou a do Cloudflare da PEN2 (04/11/2026, segundo GET de
`ies login`; também na PEN3, no POST da pesquisa com termo só de
dígitos): `<title>Attention Required! | Cloudflare</title>` e
"Sorry, you have been blocked". É reconhecimento de **tela** (como o da tela de login), feito
na camada de transporte para todos os comandos Web
(`src/transporte/bloqueio_de_seguranca.rs`), não leitura da mensagem para
decidir tratamento. Expõe `status_http` 403 e a mensagem saneada com o
código de bloqueio (não é segredo; URLs e cookies saem redigidos). A
sessão continuou válida: não há relogin, retry nem pausa automática. O
403 sem essa página continua `sessao_nao_autorizada`. Espace as execuções
ao vivo ([estado e instalações](como-o-sei-funciona/estado-e-instalacoes.md)).

## 302 do Controle para o login

`FalhaTransporte::ControleRedirecionadoParaOLogin` é o 302 do GET do
Controle (`envio_post::ler_tela_via`) cujo `Location`, resolvido contra
a instalação, é uma entrada do login dela: o SIP `/sip/login.php`, o
módulo de Goiás ou a raiz `<base>/` (`src/sessao_login/entrada_do_login.rs`).
O subcódigo é o mesmo dos fluxos que já viam o 302 para o login
(`sessao_encerrada_rode_ies_login`), com `status_http` 302: o fato é o
mesmo, sessão encerrada. Outro host, outro caminho, fragmento, a recusa
de link sem assinatura, outro 3xx ou 302 sem `Location` continuam
`Redirecionamento` (`redirecionamento_recusado`). O 302 nunca é seguido
([decisão](decisoes/sessao-login/relogin-de-leitura.md#revisão-302-do-controle-para-o-login)).

## Anexo do documento externo

`ErroAnexo` (`src/anexo_documento.rs`, Refs #60 fatia 3) tem uma
variante por fato da resposta de `documento_download_anexo`: sem tipo,
tipo fora da lista observada, cabeçalho repetido, `Content-Disposition`
fora da forma, acima do limite, vazio, exceção do SEI (com status 200 e a
mensagem saneada como apresentação) e as falhas de gravação do parcial.
Status, rede e charset continuam `FalhaTransporte`; a tela de login, a
sessão encerrada de `processo NUP`. Lista no
[contrato](../contracts/cli/documento-obter-externo.md).

## Pesquisa em documentos

`ies documento buscar` ([contrato](../contracts/cli/documento-buscar.md))
reaproveita as variantes de `processo buscar` e acrescenta, cada uma pelo
fato verificável: na tela, `ErroRegraDoFormularioDePesquisa` com a opção
`#optDocumentos` (`opcao_de_documentos_*`), as caixas de tipo
(`caixa_de_tipo_de_documento_ausente_na_tela`,
`caixas_de_tipo_de_documento_repetidas_na_tela`,
`caixa_de_tipo_de_documento_sem_valor_na_tela`,
`nenhum_tipo_de_documento_marcado_na_tela`) e
`assinante_oculto_preenchido_na_pesquisa_em_documentos`; no 302,
`ErroRedirecionamentoDaPesquisa::IdDocumento*`
(`id_documento_ausente_no_redirecionamento`, `…_repetido_…`,
`…_nao_decimal_…`); na linha, `ErroDocumentoDaPesquisa`, que embute a
recusa comum do link do processo (`ErroResultadoPesquisa`) em vez de
repeti-la. O termo inválido é `TermoDeDocumentoInvalido`
(`termo_de_documento_vazio`, `termo_de_documento_contem_controle`).

## Concluir o processo

`ErroConcluir` (`src/processo_concluir/erro.rs`) separa o que impede a
escrita (abertura, situação não lida, barra, tela, função, derivação:
saída 1, nada enviado) do resultado depois dela: `ConclusaoNaoAplicada`
(a leitura ainda lista a unidade) e `EfeitoIncertoNaConclusao` (saída 3,
com `VerificacaoPorLeitura`). `FalhaDoEnvio` classifica a resposta não
esperada da escrita sem decidir o efeito: quem decide é a leitura, uma vez,
sem retry ([contrato](../contracts/cli/processo-concluir.md)).

## Versões de documento (Refs #88)

`ErroDocumentoObter` ganhou `BarraDoDocumento(ErroBarraDoDocumento)`
(`barra_do_documento_fora_da_forma_observada`,
`barras_do_documento_repetidas_na_arvore`,
`link_das_versoes_repetido_na_barra`, `link_das_versoes_nao_reconhecido`,
`link_das_versoes_de_outro_documento`),
`ListaDeVersoesNaoPublicadaNaArvore`
(`lista_de_versoes_nao_publicada_na_arvore`), `Versoes(ErroVersao)`
(os subcódigos do leitor da lista, inclusive `versao_nao_observada`) e
`VersaoServidaDiferenteDaPedida` (`versao_servida_diferente_da_pedida`).
Com #207, `ConteudoComLinkAssinado` (`conteudo_com_link_assinado`) e, sem
rodapé nem lista, `ListaDeVersoesNaoPublicadaNaArvore` também recusam a
foto de `--saida`. A gravação local é `ErroCopia`
(`src/copia_documento.rs`): `foto_local_divergente`,
`gravacao_foto_falhou` e, na prévia offline de `salvar`,
`base_versao_ausente`, `versao_editada_ausente` e
`versao_editada_igual_a_base` (a prévia não confere a base com o SEI)
([contrato](../contracts/cli/documento-obter-versoes.md)).

## Reabrir o processo

`ErroReabrir` (`src/processo_reabrir/erro.rs`) segue a mesma divisão: o
que impede a escrita (abertura, situação não lida, barra, tela do nó,
cookie: saída 1, nada enviado) e o resultado depois dela,
`ReaberturaNaoAplicada` (a leitura não lista a unidade) e
`EfeitoIncertoNaReabertura` (saída 3). A resposta não esperada é a mesma
`FalhaDoEnvio` de concluir, com a variante `RedirecionamentoNaReabertura`;
a verificação é a mesma `VerificacaoPorLeitura`
([contrato](../contracts/cli/processo-reabrir.md)).

## Assinar o documento (Refs #174)

`ErroAssinar` (`src/documento_assinar/erro.rs`) separa o que impede a
escrita (abertura, credenciais, `ErroAssinaturaNaArvore` da ação e da
caneta, tela do documento, `ErroLinkDeAssinar`, `ErroTelaDeAssinatura`,
`cargo_nao_oferecido_pela_tela`, `documento_assinado_por_outro_usuario`,
derivação: saída 1, nada enviado) do resultado depois dela:
`AssinaturaNaoAplicada` (a caneta relida é a de antes) e
`EfeitoIncertoNaAssinatura` (saída 3, com `VerificacaoDaAssinatura`).
`FalhaDoEnvio` da assinatura (`redirecionamento_apos_assinar`,
`resposta_da_assinatura_sem_finalizacao` ou o erro do primitivo) não
decide o efeito: a leitura decide, uma vez, sem retry
([contrato](../contracts/cli/documento-assinar.md)).

## Enviar o processo

`ErroEnviar` (`src/processo_enviar/erro.rs`) segue a divisão de concluir:
antes do POST (abertura, situação, barra `ErroEnviarNaBarra`, tela
`ErroTelaDeEnvio`, consulta `FalhaDaConsulta`, destino não oferecido ou
repetido, já aberto no destino, derivação: saída 1, nada enviado) e depois
dele: `EnvioNaoAplicado` (sem o destino), `SituacaoDaOrigemDiferenteDaPedidaAposEnvio`
(o destino abriu, mas a unidade da sessão não ficou como `--manter-aberto`
pediu) e `EfeitoIncertoNoEnvio` (saída 3). A resposta não esperada é a
`FalhaDoEnvio` de concluir, com `RedirecionamentoNaoReconhecidoAposEnviar`
e `RespostaDoEnvioSemRedirecionamento`
([contrato](../contracts/cli/processo-enviar.md)).

## Configuração local: causas tipadas (#212)

`ler_configuracao_local` devolve `Result`, com causas internas
separadas para ausência, metadados, link simbólico, não regular, tamanho
excessivo, permissão aberta, leitura, JSON e campos inválidos. Não inclui
caminhos nem conteúdo no erro ou na saída pública.

O subcódigo legado `arquivo_configuracao_nao_pode_ser_lido` permanece
para arquivo ausente, não regular, excessivo ou ilegível. Novas causas
com remediação própria: `arquivo_configuracao_link_simbolico`,
`arquivo_configuracao_permissoes_inseguras`,
`arquivo_configuracao_json_invalido` e
`arquivo_configuracao_campo_invalido`. O campo inválido inclui
caminho ausente, não textual, vazio ou relativo. Não há fallback
para variáveis individuais quando `IES_CONFIG_FILE` é informado.
A verificação usa `symlink_metadata` antes da abertura e confere
o arquivo aberto, sem consumir ou registrar dados de sessão.

## Gravação privada local (#211)

Toda gravação de arquivo privado passa por `crate::arquivo_privado`,
que devolve uma falha tipada sem caminho nem conteúdo (destino já
existente, destino não regular, diretório ausente, diretório que é link
simbólico, limite, criação, escrita, publicação). Cada módulo a traduz
para os subcódigos que já tinha; nenhum mudou de nome
([decisão](decisoes/arquivos-privados/gravacao-privada.md#migração)).

Um subcódigo novo: `diretorio_do_arquivo_da_caixa_link_simbolico`, da
exportação da caixa (`--saida` de `unidade listar-processos`, `recebidos`
e `gerados`), quando um ancestral do destino é link simbólico. Antes a
exportação seguia o link. Na sessão, o mesmo caso sai como
`destino_da_sessao_nao_eh_arquivo_regular`; na configuração, como
`configuracao_ja_existe_ou_sem_permissao`; no texto, no anexo e na cópia,
pelos subcódigos de diretório link simbólico que eles já tinham.
