---
type: CommandContract
title: Iniciar processo pelo cadastro Web do SEI
operation: processo.criar
status: proposed
command: ies processo criar --tipo TIPO --especificacao TEXTO [--confirmar]
effect: write-process-state
implementation: partial
---

# `ies processo criar` — contrato proposto (#245)

```text
ies processo criar --tipo TIPO --especificacao TEXTO
  [--interessado TEXTO ...] [--nivel publico|restrito]
  [--hipotese-legal ID#NOME] [--confirmar]
```

A unidade é a unidade ativa da sessão, sujeita a `--unidade`
global se for conferível na tela. O padrão é acesso público.
Sem `--confirmar` é **prévia**, não escrita. A lista publicada
deve conter exatamente uma entrada do tipo pedido, e o cadastro
aberto deve publicar o mesmo ID. Não inferir tipo ausente de
uma lista apenas dos mais usados: expandir a lista completa
pela ação da tela ou recusar a observação incompleta.

## Prévia

Ler a escolha e o cadastro sem enviar o formulário de criação.
Conferir os campos publicados, o assunto sugerido, o nível
habilitado e, no restrito, a hipótese legal oferecida.
Resolver interessados pedidos na própria tela; recusar
ambiguidade. A especificação é o texto fornecido, nunca um
valor construído por script executado. Exibir o que será
enviado e a contagem de requisições reais.

A prévia e o `--confirmar` verificam o atributo `maxlength` do
`input txtDescricao` **na tela servida**. Se o comprimento em unidades
UTF-16 exceder o máximo publicado, recusam antes do POST criador:
`ErroCadastro::EspecificacaoAcimaDoLimite`, saída 1, sem escrita.
JSON publica `limite_da_especificacao` e
`comprimento_da_especificacao_utf16`, não o texto do usuário;
a saída humana indica ambos os números. Se o atributo não existir,
não se supõe limite global. Um `maxlength` presente mas ilegível
recusa como HTML não comprovado, sem POST. Nas capturas
`procedimento_gerar.html` (SEI 4.0.9) e
`procedimento_gerar_go_5_0_3.html` do Todos, o máximo observado é
100, mas não é constante gravada no IES.

`--nivel restrito` sem hipótese legal selecionável **não**
autoriza fallback silencioso para público. Na consulta AJAX publicada pelo cadastro restrito, quando houver
mais de uma hipótese, o usuário pode informar
`--hipotese-legal ID#NOME` com os dois valores literais
observados. O IES exige correspondência exata e única na resposta
daquela mesma sessão, sem selecionar o primeiro item. Se não houver
escolha e a lista for múltipla, recusa antes da escrita. Não se pode
informar a opção com `--nivel publico`; o ID é opaco.
O seletor explícito ainda não se aplica a uma opção fornecida
diretamente no HTML sem AJAX: nessa situação, omita a flag ou
aguarde validação específica. Não extrapolar para instalações
que não publicaram essa consulta.

## Confirmação

Uma única submissão do cadastro publicado; sem retry nem
redirect automático. A resposta esperada é um 302 para
a árvore com `id_procedimento`, cuja identidade deve ser
confirmada em leituras novas de tela e árvore, associada a um NUP não vazio. Uma terceira leitura, das propriedades oferecidas pela barra do processo, deve demonstrar tipo, especificação e nível de acesso iguais aos solicitados. Resposta 302 e NUP sem essas propriedades não permitem publicar `confirmado: true` (saída 1; sem retry).
A resposta sozinha nunca prova a criação. No `Location`, aceitar
**uma única** ocorrência da ação `procedimento_trabalhar` e **um único**
`id_procedimento` numérico; quaisquer repetições, mesmo com um
parâmetro inválido, são ambiguidade e resultam em
`ErroCriar::NaoConfirmado` (saída 1), sem seguir a URL assinada.
Na escolha de tipo por link, aplicar a mesma regra a `acao` e
`id_tipo_procedimento` antes de qualquer GET.

Falha definitivamente anterior ao envio: saída 1 (ou 2
para argumentos inválidos). Depois de iniciar a escrita,
qualquer resultado que não permita estabelecer a identidade
e o efeito sai com `FalhaNaCriacao { causa, requisicoes }` (causa
`ErroCriar::NaoConfirmado` ou a etapa que falhou), saída 1, sem repetir POST.
O usuário deve conferir a unidade antes de nova tentativa.

## Subcódigos do leitor de tipos

| Erro (stderr) | Fato verificável |
| --- | --- |
| `ErroListaDeTipos::ListaDeTiposHtmlInvalido` | HTML da lista não pode ser interpretado |
| `ErroListaDeTipos::TabelaDeTiposAusenteOuRepetida` | Tabela de tipos ausente, repetida ou não é tabela |
| `ErroListaDeTipos::TipoNaoOferecido` | Tipo não consta da lista completa |
| `ErroListaDeTipos::TipoDeProcessoAmbiguo` | Mais de uma âncora com nome exato |
| `ErroListaDeTipos::TipoSemAcao` | Âncora sem ação segura ou desabilitada |
| `ErroListaDeTipos::ListaIncompleta` | Tipo não encontrado na lista curta; não inferir ausência |
| `ErroCadastro::CadastroDeProcessoHtmlInvalido` | HTML do cadastro ilegível |
| `ErroCadastro::CadastroDeProcessoAusenteOuRepetido` | Formulário de cadastro ausente ou repetido |
| `ErroCadastro::TipoDeCadastroDivergente` | ID do tipo no cadastro diferente do escolhido |
| `ErroCadastro::EspecificacaoNaoPublicadaNoCadastro` | Campo de especificação ausente ou desabilitado |
| `ErroCadastro::EspecificacaoAcimaDoLimite` | Especificação maior que `maxlength` observado no cadastro; rejeita prévia e confirmação antes do POST |
| `ErroCadastro::LimiteDaEspecificacaoInvalidoNoCadastro` | Atributo `maxlength` presente, mas não numérico; HTML insuficiente para autorizar escrita |
| `ErroCadastro::AssuntoSugeridoNaoUnico` | Nenhum ou vários assuntos selecionáveis no cadastro |
| `ErroCadastro::AssuntoDesabilitado` | O `select` de assunto está desabilitado: a opção visível não comprova escolha habilitada; recusa antes do POST |
| `HipoteseLegalDesabilitada` | `select` da hipótese legal está desabilitado quando o nível solicitado é restrito; recusa antes do POST |
| `ErroCadastro::NivelNaoOferecido` | Rádio de acesso solicitado não está publicado |
| `ErroCadastro::NivelDesabilitado` | Rádio publicado mas desabilitado |
| `HipoteseLegalNaoIdentificada` | Hipótese legal única não publicada |
| erro do clap (saída 2) | Tipo CLI vazio ou contém controle |
| erro do clap (saída 2) | Especificação CLI vazia ou contém controle |
| erro do clap (saída 2) | Interessado CLI vazio ou contém controle |

## Subcódigos candidatos

Os nomes abaixo são reservados **como proposta**, ainda sem
implementação Rust e sem afirmar aprovação do verificador:

- `ErroListaDeTipos::TipoNaoOferecido`
- `tipo_de_processo_ambiguo`
- `ErroListaDeTipos::ListaIncompleta`
- `cadastro_de_processo_desconhecido`
- `assunto_sugerido_nao_unico`
- `ErroCadastro::NivelNaoOferecido`
- `HipoteseLegalNaoIdentificada`
- `ErroInteressado::Ambiguo`
- `criacao_de_processo_recusada`
- `ErroCriar::NaoConfirmado`

Subcódigos já emitidos pelo leitor offline da lista de tipos:
`lista_de_tipos_html_invalido`, `tabela_de_tipos_ausente_ou_repetida`,
`ErroListaDeTipos::TipoNaoOferecido`, `tipo_de_processo_ambiguo`,
`ErroListaDeTipos::TipoSemAcao`, `ErroListaDeTipos::ListaIncompleta`.

Esses subcódigos precisam de tipos reais e integração a
`docs/erros-tipados.md` antes de serem publicados na CLI.


## Subcódigos do fluxo Web

| Erro (stderr) | Recusa |
| --- | --- |
| `ErroCriar::Cadastro` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::NaoConfirmado` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ProvaDaCriacao::PropriedadesDivergentes` | Tipo, especificação ou nível lido nas propriedades não corresponde ao POST; escrita não confirmada, saída 1 |
| `ErroCriar::PosEscrita { correspondentes: Some(nups), .. }` com dois ou mais NUPs | Mais de um candidato comprovadamente compatível após POST incerto; não atribuir autoria nem repetir |
| `ErroCriar::PosEscrita { correspondentes: Some([nup]), .. }` | Um único candidato novo tem propriedades iguais, mas POST não respondeu: não há vínculo causal; saída 1, possível resíduo, sem retry |
| `ErroCriar::Transporte` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::Formulario` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroMenu::LinkDeIniciarProcessoAusente` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroMenu::LinkDeIniciarProcessoNaoReconhecido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroMenu::LinkDeIniciarProcessoRepetido` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::Lista` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::Menu` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::TipoSemIdentificador` | Etapa observada não satisfeita; não repetir operação automaticamente |
| `ErroCriar::UnidadeDiferente` | Etapa observada não satisfeita; não repetir operação automaticamente |


## Subcódigos do interessado

| Erro (stderr) | Recusa |
| --- | --- |
| `ErroInteressado::InteressadoControlesAusentes` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `ErroInteressado::AutocompletarNaoDeclarado` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `ErroInteressado::DerivacaoRecusada` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `ErroInteressado::InteressadoRespostaNaoReconhecida` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `ErroInteressado::NaoOferecido` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |
| `ErroInteressado::Ambiguo` | Autocomplete ou identificação do interessado não demonstrável pela tela servida |

## Subcódigos da hipótese legal carregada por AJAX

| Erro (stderr) | Fato observado |
| --- | --- |
| `ErroHipotese::HipoteseLegalAjaxNaoDeclarada` | Cadastro restrito publicou o select pendente mas não declarou uma única URL AJAX literal |
| `ErroHipotese::HipoteseLegalAjaxNaoDerivada` | URL AJAX ou corpo não puderam ser derivados com segurança pela infraestrutura compartilhada |
| `ErroHipotese::HipoteseLegalAjaxRespostaNaoReconhecida` | Consulta não devolveu XML 200 com um select legível e opção válida; também recusa uma opção única ou escolha explícita com ID/nome não publicável (URL ou caracteres de controle) |
| `ErroHipotese::OpcaoNaoUnica` | XML da consulta não ofereceu exatamente uma hipótese legal efetiva habilitada |

Se o próprio cadastro traz exatamente uma hipótese legal habilitada, não
se faz AJAX. Se o select só traz `null`, consulta-se a URL assinada
publicada na tela, uma vez, por POST **sem efeito de escrita**.
A resposta com múltiplas opções não permite escolher por posição:
sem `--hipotese-legal`, recusar antes do POST criador. Quando as
opções efetivas forem distintas, habilitadas e imprimíveis, a recusa
`ErroHipotese::OpcaoNaoUnica` apresenta os pares realmente lidos:
no JSON, `hipoteses_legais_oferecidas` contém objetos
`{id, nome, argumento}`, sendo `argumento` o literal `ID#NOME`;
na saída humana, mostra cada `ID#NOME`. Não publica URL assinada,
cookie nem hipótese não habilitada. IDs duplicados, nomes com controles
ou opções ausentes não geram lista de sugestões. O resultado continua
sendo **saída 1, sem escrita**: uma nova prévia explícita é necessária
antes de qualquer confirmação. A consulta AJAX
usa os quatro campos documentados pela observação da interface
e soma **uma** requisição à prévia e à confirmação restritas.

## Orçamento e evidência

Argumento rejeitado sem rede: 0. Prévia: consultas necessárias
para abrir lista completa, escolher tipo e conferir o cadastro,
incluindo AJAX somente quando o pedido o exigir; zero POST de
**criação**. Confirmação: mesmas consultas, **um** POST criador
e leitura nova de verificação. O número exato de requisições
por versão ainda precisa de teste replay e medição, não de
estimativa. O POST que apenas escolhe o tipo em SEI 4.1+
não é a escrita criadora. Nos testes com transporte injetado,
a prévia 4.0 tem **3 GET**; a confirmação 4.0 tem **6 pedidos**
(5 GET e 1 POST criador);
a confirmação 4.1 tem **7 pedidos** (5 GET, 1 POST de seleção
e 1 POST criador). Não extrapolar esses totais a instalações reais
sem observação e replay. Em URLs ambíguas recusar antes da próxima
requisição; depois de iniciar a escrita, saída 1 sem retry.

[Observação](../../docs/como-o-sei-funciona/inicio-do-processo.md)
e [decisão](../../docs/decisoes/processo-criar.md).

## Seleção explícita da hipótese na resposta AJAX (#245)

A execução da [cassette pública da UnB](../../docs/validacao-replay/2026-11-05-processo-criar-acervo.md)
encontrou duas respostas com opções múltiplas. O parâmetro
`--hipotese-legal ID#NOME` só é aceito para acesso **restrito**
e usa ID e nome **exatamente** observados no XML da consulta
`hipotese_legal_select_nome_base_legal`, executada uma vez.
Opções desabilitadas, ausentes, com ID duplicado ou nome divergente
não legitimam a escrita. Nunca executar JavaScript.
A prévia publica o ID escolhido, e o POST criador só pode utilizar
o ID já conferido. Prévia com AJAX: **4 requisições** sintéticas
(3 GET + 1 POST de consulta, nenhum POST criador).
Confirmação com AJAX: **7 requisições** (5 GET + 1 POST de consulta
+ 1 POST criador, seguidos de duas leituras verificadoras).
As contagens ao vivo permanecem NOT_RUN até ensaio autenticado.

| Erro (stderr) | Fato observável |
| --- | --- |
| erro do clap (saída 2) | ID#NOME ausente/inválido ou escolha em nível público |
| `ErroHipotese::EscolhaNaoOferecida` | Par exato ID#NOME não consta do AJAX observado |
| `ErroHipotese::EscolhaInlineNaoVerificada` | Usuário exigiu escolha explícita sem consulta AJAX que a comprove |

## Reconciliação depois do POST

As regras de comparação da caixa Gerados, classificação de candidatos,
recusa expressa da Infra, HTTP 4xx inconclusivo e orçamentos por cenário
estão no [contrato de reconciliação](processo-criar/reconciliacao.md).

## Catálogo de subcódigos da reconciliação

O significado e as condições de emissão permanecem no
[complemento de reconciliação](processo-criar/reconciliacao.md).

| Erro (stderr) | Fato verificável |
| --- | --- |
| `ErroFotografia::CoberturaIncompleta` | Total ou faixa insuficiente para atestar todos os Gerados |
| `ErroFotografia::IdentidadeAlterada` | Um ID pré-existente passou a apontar outro NUP |
| `ErroFotografia::IdentidadeAnteriorAusente` | Um ID pré-existente desapareceu da nova lista |
| `ErroConferencia::LeituraAusente` | Novo candidato sem leitura de processo NUP |
| `ErroConferencia::LeituraRepetida` | Mesma identidade conferida duas vezes |
| `ErroConferencia::IdentidadeDivergente` | Par ID/NUP da leitura diferente do candidato |
| `ErroConferencia::PropriedadesNaoObservadas` | Tipo ou especificação não publicados na leitura |
| `LinkDoCandidatoNaoComprovado` | Link da linha Gerados ausente, fora da origem ou com identidade divergente |
| `LinksDoCandidatoRepetidos` | Dois links para o mesmo par de Gerados |
| `ErroCriar::PropriedadesNaoOferecidas` | A árvore conferida não publicou endereço de propriedades |
