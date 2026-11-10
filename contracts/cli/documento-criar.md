---
type: CommandContract
title: Criar documento interno no processo pelo cadastro Web
operation: documento.criar
command: ies documento criar --processo NUP --tipo TIPO [--descricao TEXTO] [--nivel NIVEL] [--confirmar]
effect: write
implementation: partial
source_commit: 323eb01
---

# Criar documento interno — issue #246

## Limite

Operação planejada: `ies documento criar --processo NUP --tipo TIPO
[--descricao TEXTO] [--nivel publico|restrito|sigiloso] [--confirmar]`.
O valor `--nivel` e eventuais hipóteses legais devem ser conferidos
contra o cadastro servido; não há autorização para supor uma seleção
implícita. A operação **não** grava texto no editor e **não** assina.

Estado `partial`: a CLI oferece prévia nas telas com link direto e assinado para o cadastro (`documento_gerar`) **e** nas telas com `escolher(ID)` quando a função servida comprova duas instruções literais: atribuir `hdnIdSerie` e submeter `frmDocumentoEscolherTipo`. Nessa segunda forma, faz um POST **de navegação sem criação** para `documento_escolher_tipo`, valida a Location 302 assinada do mesmo processo e faz um GET do cadastro; não executa JS, não segue redirecionamentos automaticamente e não repete requisições. A confirmação tem um único POST **de escrita** e reconciliação por releitura, somente com `--nivel publico` explícito, controle Salvar e formulário observados. Até CI, replay e ensaio live, tudo permanece WIP, sem validação para uso real. Níveis restrito/sigiloso recusam antes de escrever, pois dependem de hipótese legal não resolvida.

## Prévia

Abrir o processo, confirmar identidade e unidade da sessão, constatar
que o processo está aberto nessa unidade e seguir o link **publicado**
"Incluir Documento" (`documento_escolher_tipo`). Ler a lista
`tblSeries`, recusar nomes ausentes e ambíguos, ignorar "Externo",
abrir o cadastro do tipo exato (GET direto ou POST de navegação seguido de GET) e conferir `lblSerieTitulo`, formulário,
campos e nível solicitado. A prévia não envia POST de criação. Uma
eventual leitura por POST (`hdnFiltroSerie=T`) exige declaração de
efeito separada e contagem; `documento_gerar_multiplo` não é leitura.

## Confirmação

Antes do primeiro POST de escrita, todas as recusas precisam estar
resolvidas. Derivar exatamente um formulário `documento_gerar` do
HTML servido pelo primitivo `formulario_web`, com `EfeitoAvaliado`.
Não construir URL assinada, executar JS, enviar campos fora da lista
observada, repetir POST nem fazer fallback.
Quando o cadastro exigir controles HTML `required`, recusar a escrita antes do POST se algum controle ativo não puder ser atendido pela CLI, inclusive descrição vazia. Controles `hidden` e desabilitados não exigem escolha do usuário. Antes da tentativa, recusar sem POST uma árvore inicial incompleta, um processo sem ID ou IDs de documentos repetidos ou vazios. Depois da tentativa,
reabrir o processo e conferir a diferença entre as árvores.

Uma escrita comprovada exige **exatamente um novo documento** (id
único, número SEI publicado, tipo igual ao solicitado) e a preservação
dos documentos anteriores. O `id_documento` da resposta não substitui
o número SEI da árvore. Se o POST puder ter sido enviado e a verificação
falhar, ou se a mudança for ambígua, saída 1
(`ErroPreviaCriacao::NaoConfirmadoNoEnvio`/`NaoConfirmadoNaReleitura`,
com o contexto `TentativaDeCriacao { requisicoes }`).

## Recusas tipadas

| Variante ou `onde` (stderr) | Critério |
| --- | --- |
| `ErroPreviaCriacao::ProcessoNaoAbertoNaUnidade` | A situação conferida não inclui a unidade ativa |
| `ErroCriar::TipoAusente` | Não há opção exata na lista completa |
| `ErroCriar::TipoAmbiguo` | Mais de uma opção com o mesmo nome |
| `ErroCriar::ListaForaDaForma` | `tblSeries` ou links não interpretáveis |
| `ErroCriar::CadastroForaDaForma` | Cadastro ou tipo aberto inconsistente |
| `ErroCriar::NaoAplicado` | Verificação concluiu que não surgiu documento |
| `ErroPreviaCriacao::NaoConfirmadoNoEnvio / NaoConfirmadoNaReleitura` | Resposta ou árvore não permite conferir unicidade e identidade |
| `ErroLinkIncluirDocumento::BarraForaDaForma` | A barra da raiz não tem estrutura interpretável |
| `ErroLinkIncluirDocumento::BarraRepetida` | A barra da raiz aparece duas vezes |
| `ErroLinkIncluirDocumento::LinkRepetido` | A barra publica mais de uma ação de incluir |
| `ErroLinkIncluirDocumento::EnderecoNaoReconhecido` | O link de inclusão não é a URL relativa assinada esperada |
| `ErroLinkIncluirDocumento::EnderecoDeOutroProcesso` | O link de inclusão aponta para outra identidade |
| `ErroPreviaCriacao::LinkNaoPublicado` | A unidade não publicou a opção de incluir |
| `ErroPreviaCriacao::UnidadeNaoPublicada` | A tela não identifica a unidade de trabalho |
| `ErroPreviaCriacao::SituacaoNaoLida` | A árvore não publicou a situação segura para gravação |
| `ErroCriar::LinkDoTipoNaoNavegavel` | O onclick/script não comprova exatamente a navegação segura por formulário |
| `ErroCriar::EnderecoDoTipoNaoReconhecido` | URL do tipo ausente, inválida ou vinculada a outro processo |
| `ErroCriar::TipoDoCadastroDiferente` | A tela aberta não identifica o tipo pedido |
| `ErroCriar::DescricaoNaoOferecida` | A descrição solicitada não tem controle utilizável |
| `ErroCriar::NivelNaoOferecido` | O nível solicitado não aparece habilitado |
| `ErroCriar::CampoObrigatorioNaoAtendido` | O cadastro publica controle HTML obrigatório que a CLI não consegue preencher com valor observado |
| `ErroPreviaCriacao::TelaRedirecionada` | A tela de leitura devolveu 302 não seguido |
| `ErroPreviaCriacao::NivelNaoInformado` | Confirmação exige --nivel publico explícito; não presumir nível |
| `ErroPreviaCriacao::NivelRestritoSemHipotese` | Nível não público exige resolver a hipótese legal antes da escrita |
| `ErroPreviaCriacao::SubmissaoNaoComprovada` | Cadastro não contém a regra observada de submissão |
| `ErroPreviaCriacao::ArvoreIncompleta` | Pasta fechada impede comparação de documentos |
| `ErroPreviaCriacao::FotoInicialInconsistente` | Antes do POST, a árvore contém IDs de documento repetidos/vazios ou ID de processo ausente; nenhuma escrita é enviada |
| `TipoDeDocumentoVazio` | O tipo informado está vazio |
| erro do clap (saída 2) | Falha de validação de entrada |


Erros de abertura, atributos, transporte e formulário sobem como causa
tipada. Entrada inválida sai 2 (clap); recusa antes do POST sai 1;
sucesso demonstrado sai 0; escrita não confirmada após o POST sai 1.

## Saída projetada

A prévia e a confirmação exibem `descricao` e `nivel` quando informados
na operação: no JSON, os campos são `string` ou `null`; na apresentação
humana, aparecem como "Descrição" e "Nível" quando presentes. A prévia
permite conferir esses dados **antes** de enviar o único POST criador;
não altera o formulário nem acrescenta requisições.

Prévia: `{"nup":"…","tipo":"Despacho","confirmado":false,
"alterado":false,"requisicoes":N}`.
Após escrita comprovada: `{"nup":"…","tipo":"Despacho",
"confirmado":true,"alterado":true,"numero_sei":"…",
"requisicoes":N}`. Na saída 1 de escrita não confirmada, `requisicoes`
informa as tentativas realmente contadas (inclusive POST e releitura);
não se interpreta a contagem como prova de aceite. Após a escrita, uma
falha da leitura verificadora mantém saída 1, preserva a causa tipada
em `causa_releitura` e publica `status_http` e
`mensagem_do_servidor` quando observados. O POST não é repetido;
`causa_envio` permanece reservado às falhas do próprio envio.
Nenhum segredo ou URL assinada na saída.

## Evidências e validação

[Observação](../../docs/como-o-sei-funciona/inclusao-documento-interno.md)
e [decisão](../../docs/decisoes/documento-criar.md).
Oráculo: Todos `src/todos/documentos.py` (`criar`,
`_criar_na_arvore`, `cadastrar_da_tela`, `_depois_de_enviado`).
Cassettes Goiás `documento-incluir` e `documento-escritas`.
O replay GO das duas cassettes é **obrigatório no CI**: ZIP e membros conferidos por hash; 15 respostas-alvo (3 lidas, 5 recusadas e 7 não aplicáveis). [CI 42127145672](https://github.com/franklinbaldo/ies/actions/runs/42127145672): PASS em prek, Release e replay (head `6603a36`).

Os testes sintéticos verificam prévia de 5 GET/0 POST e confirmação de 6 GET/1 POST, com releitura e escrita não confirmada sem retry. [RED 42120924295](https://github.com/franklinbaldo/ies/actions/runs/42120924295): 1 teste falhou antes da guarda de nível pré-rede. Um teste de 302 sem Location apresentou [RED 42131591459](https://github.com/franklinbaldo/ies/actions/runs/42131591459) (1 falha, 1.610 aprovados); a correção exige Location não vazia para reconhecer o 302 como resposta conferível. O head corrigido requer CI próprio. RED e compilação Rust **locais: NOT_RUN** (cargo/rustc e DNS indisponíveis).

**Live:** login, prévia, escrita e releitura em treinamento **NOT_RUN**, 0 requisições, 0 POSTs e nenhum resíduo nesta sessão. A escrita está autorizada só em treinamento; falta a sequência prévia observada → uma única confirmação → leitura independente, com número SEI e resíduo registrados.

## Guarda de script e causas da retomada 🐙

A confirmação não pode inferir a instrução de submissão de uma
substring arbitrária do HTML. Exige uma única atribuição literal
observada de `hdnFlagDocumentoCadastro` em `script` ativo;
`template`, `script type=application/json`, comentário de bloco
e script duplicado não autorizam a escrita. Os quatro casos são
testados com transporte injetável e exatamente 5 GET / 0 POST.
Formas não reconhecidas recusam com
`ErroPreviaCriacao::SubmissaoNaoComprovada` antes do POST.

As causas da prévia, escolha e barra de inclusão são compostas por
`thiserror` (`#[from]`), sem mapeamento manual de subcódigos. A saída
legada mantém as folhas de erro publicadas, com caminho tipado
interno e preservação da mensagem/status HTTP quando observados.
A escrita incerta continua saída 1.


A guarda de confirmação também recusa `script[src]`, cujo corpo inline
não é executado pelo navegador, e atribuições presentes apenas em
literais JavaScript multilinha. Esses casos têm testes sintéticos com
**5 GET, 0 POST criador**: formas inertes não autorizam escrita.

## Fronteira de Location da confirmação

O retorno 302 somente pode contribuir para uma confirmação se a
`Location` publicada permanecer no controlador HTTPS da mesma instalação,
com uma ação explícita e única. Um 302 sem Location, com host externo,
com caminho alheio ou sem ação não comprova atribuição causal da mudança
na árvore: escrita não confirmada (saída 1), mesmo quando se observa um novo
documento. A leitura de reconciliação ocorre uma vez após o POST;
não seguir Location e não repetir a escrita.

## Forma conservadora do script antes da escrita

O campo oculto de cadastro só é reescrito se a tela publicar exatamente
uma forma de código reconhecida; não basta encontrar a atribuição como
substring. A captura Todos `documento_gerar.html @323eb01` contém
`function submeter()` com desabilitação do botão, sinalizador igual a
`2` e envio direto de `frmDocumentoCadastro`, nessa ordem.
O leitor exige a sequência literal direta **completa**; a atribuição
isolada, inclusive em um script sintético mínimo, não comprova a
submissão e é recusada antes da escrita. Também recusa `if(false)`,
funções não relacionadas, aninhamentos, strings, templates, comentários,
`script[src]` e redefinições ambíguas. Forma não observada:
`ErroPreviaCriacao::SubmissaoNaoComprovada`, **antes do POST**.

RED observado no [CI 42158719053](https://github.com/franklinbaldo/ies/actions/runs/42158719053):
1 teste novo falhou, 1.659 passaram, 18 ignorados; orçamento adversarial
**5 GET, 0 POST**. Prova live de criação em treinamento continua
pendente, sem qualquer permissão para repetir escrita incerta.


## Vínculo do controle Salvar (06/11/2026)

A função `submeter()` isolada no código servido não demonstra que o
cadastro ofereceu uma ação de criação. A captura SEI 4.0.9 do Todos
`documento_gerar.html @323eb01` publica dois controles `button#btnSalvar`
(id repetido no topo e no rodapé), ambos `type=button`, habilitados,
`name=btnSalvar` e `onclick=confirmarDados()`. Esta ligação é
conferida **antes do POST** juntamente com a função `submeter()` completa.
Sem botão, com tipo/callback divergente, desabilitado ou com outro botão
`btnSalvar` contraditório, a confirmação recusa com
`ErroPreviaCriacao::SubmissaoNaoComprovada`, após os **5 GET de prévia e
sem POST criador**, nos testes sintéticos. A função JS não é executada;
atributos são lidos por `html_atributos`. Testes positivos aceitam dois
botões iguais, como na captura. Além do atributo `disabled` direto,
`hidden` no botão ou em ancestral e `fieldset disabled` que o contém
recusam a escrita: 5 GET de prévia, 0 POST criador (casos adversariais
sintéticos; não atribuídos ao SEI real). Isso ainda não comprova live
em treinamento.

**RED do novo teste antes da implementação:** [CI 42170278378](https://github.com/franklinbaldo/ies/actions/runs/42170278378),
**1 falha nova, 1.683 testes aprovados, 18 ignorados**. O teste
`salvar_sem_botao_observado_ou_com_acionador_divergente_recusa_antes_do_post`
excedeu o roteiro de cinco GET na guarda anterior; nenhuma requisição real.

## Prova da cadeia acionável (06/11/2026)

O botão `btnSalvar` e uma função `submeter()` separadamente não
autorizam a escrita. A guarda exige uma única `confirmarDados()`
com chamada `submeter()` dentro de `if (OnSubmitForm())`,
além de `submeter()` completo. Recusas por cadeia ausente, alterada
ou duplicada geram `ErroPreviaCriacao::SubmissaoNaoComprovada` antes do
POST. Testes adversariais: 5 GET/0 POST; confirmação positiva: 6 GET,
1 POST e verificação por leitura. Não se executa JS.

## Aceite identificável após o POST (06/11/2026)

Um HTML 200 genérico, inclusive cadastro reapresentado, **não** confirma
aceite: a árvore pode ter mudado por outro agente da conta compartilhada.
A confirmação sintética aceita resposta 302 para o mesmo controlador
com ação `documento_visualizar` ou `arvore_visualizar` e `id_documento`
único e igual ao nó novo observado na releitura. A segunda forma está no
oráculo Todos `tests/test_escritas_diretas_010.py @323eb01`, não constitui
comprovação de escrita ao vivo no IES. O redirecionamento jamais é seguido.
Mesmo após recusa HTTP 403/503 ou validação Infra, sempre se realiza
a releitura independente uma vez; o efeito segue incerto (saída 1),
com `causa_envio` tipada, `status_http` e mensagem saneada quando observados.
Nenhum segundo POST é autorizado, inclusive quando a árvore mudou.
A forma efetiva do retorno do SEI em treinamento permanece a validar.

## Prova de script ativo antes dos POSTs de navegação

A seleção do tipo por `escolher(id)` e a troca do filtro por
`exibirSeries(tipo)` exigem a função literal observada publicada em
um único `script` clássico inline ativo. Texto idêntico em
`template`, comentário HTML ou `script type=application/json` não
autoriza o POST de navegação; não se executa JavaScript nem se inventa
endpoint. A recusa conserva `ErroCriar::LinkDoTipoNaoNavegavel`
e acontece antes do envio daquele formulário. Testes adversariais
sintéticos cobrem os três conteúdos inertes em ambos os caminhos e
controles positivos com código ativo. Não extrapolar para outras
versões/instalações sem observação.


## Controles de seleção ocultos (06/11/2026)

Na lista de tipos, um link `<a hidden>` ou um link dentro de
ancestral `hidden` não constitui oferta da interface: não entra em
`tipos_oferecidos` e não pode ser selecionado por `escolha_do_tipo`.
O mesmo vale para o acionador `ancExibirSeries`: somente uma cópia
oculta devolve ausência de filtro, sem POST. Uma cópia visível segue
utilizável sem ambiguidade artificial. A guarda lê atributos com
`html_atributos` e intervalos do DOM; não executa JS. A regra
`fieldset disabled` é mantida apenas para os controles do formulário,
não aplicada indistintamente aos links.

Regressões adversariais sintéticas, sem atribuir `hidden` às telas
reais: `tipos_em_controles_hidden_nao_sao_oferecidos` e
`exibir_todos_hidden_nao_autoriza_filtro`. RED antes da mudança:
[CI 42322725329](https://github.com/franklinbaldo/ies/actions/runs/42322725329),
**2 falhas novas / 1.848 aprovados / 23 ignorados preexistentes**.
A confirmação ao vivo e a atribuição causal de documento criado
continuam pendentes exclusivamente em treinamento.


## Atualização vigente — ablação da prova JavaScript (07/11/2026)
Seções históricas de análise de JS acima não são requisito de confirmação atual.
Por decisão motivada do 🦡 Texugo na PR #248, a guarda valida o botão
Salvar ativo e o formulário servido (ação e campos), sem simular callbacks
JavaScript. O servidor decide aceitar ou recusar a submissão HTTP; a CLI
preserva prévia, exatamente um POST, 302 causal, releitura independente,
número SEI e saída 1 para escrita não confirmada. Aceita-se o risco residual de
divergência entre interação no navegador e POST da CLI. Testes de equivalência
hipotética de JS foram removidos em ablação explícita e documentada.
Isso não constitui validação live da criação de documento.
