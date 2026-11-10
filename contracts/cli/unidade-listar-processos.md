---
type: CommandContract
title: Listar processos da unidade
operation: unidade.listar_processos
command: ies unidade listar-processos
effect: read
implementation: partial
source_commit: 3d4a38c3b5987e4a8fc7f97ded188c4876ed77cd
---

# Listar processos da unidade

Lê os processos da unidade ativa na tela **Controle de Processos** do SEI
Web pela [visão detalhada](unidade-listar-processos-visao-detalhada.md)
(decisão do proprietário, 02/11/2026), com as colunas do
[Nível de detalhe](unidade-colunas.md); vinda da R, a sessão fica na D,
sem restaurar (decisão do proprietário, 02/11/2026).
`ies unidade recebidos` e `ies unidade gerados` leem uma das
[listas independentes](unidade-recebidos-gerados.md) pela mesma operação. A CaixaDeEntrada é da unidade;
sessão e credenciais são meios de acesso, não entidades proprietárias.

## Configuração e limites

A sessão existente é fornecida por caminhos locais definidos
em variáveis de ambiente, **nunca por valores secretos na linha de comando**:

- `IES_URL_CONTROLE_FILE`: caminho privado contendo URL HTTPS assinada
  `procedimento_controlar` publicada pela instalação autorizada.
- `IES_COOKIE_FILE`: caminho privado com o cookie de sessão existente.
- `IES_ORIGEM_FILE`: caminho privado com a origem HTTPS autorizada.

Os arquivos devem respeitar as regras de privacidade do adaptador
(no Unix, modo 0600). O transporte deve rejeitar **links simbólicos**
para URL assinada, origem autorizada e cookie, distinguindo-os de arquivo
regular: metadados de links não provam posse segura do arquivo alvo.
Esta validação deve ocorrer antes de qualquer envio de cookie por rede. O IES confere a origem antes de ler cookies e
não segue redirects, não executa login, escritas ou tentativa de outro
backend. Os POSTs são os da troca de visão e da página da D
([visão detalhada](unidade-listar-processos-visao-detalhada.md)) e o de
`ies unidade recebidos|gerados --pagina N`
([páginas da caixa](unidade-listar-processos-paginas.md)). Não há
configuração persistente nem automação de autenticação.

`--unidade SIGLA` exige que a página identifique a unidade esperada.
`--saida ARQUIVO` exporta o mesmo array JSON da saída **somente depois**
de verificar total e faixa observada. O arquivo novo é criado sem
sobrescrever destinos existentes; em Unix recebe modo privado 0600.
Sem essa opção, a saída de terminal é preservada. O comprovante não
exibe dados dos processos ou caminhos de credenciais. Não declara
snapshot transacional.

## Pré-validação da sessão local, sem rede

`ies status --verificar-sessao-local --json` confere, **sem abrir socket**,
os três arquivos privados: regularidade/permissões, URL assinada de
`procedimento_controlar`, origem HTTPS autorizada incluindo caminho-base
e sintaxe do cabeçalho Cookie. A validação de origem antecede a leitura
do cookie, usando as mesmas regras da leitura de caixa para evitar divergência
entre prévia e transporte. Falhas são variantes de `FalhaTransporte` e
subcódigos estáveis; nenhuma entrada secreta é exibida.

Sucesso declara somente `sessao_local: formato_e_origem_verificados`,
`conexao: nao_verificada` e `acesso_remoto: false`: **não** demonstra
sessão viva, identidade, permissão, renovação de hash ou cobertura de
paginação. O modo existente `--verificar-arquivos` continua consultando
**apenas metadados**, sem ler conteúdo. Refs #57.

## Resultado e alcance

`ies unidade listar-processos` devolve um **array com uma lista** D
`categoria: null`, `colunas_ausentes` e, por processo, `categoria`
observada e `colunas` ([visão detalhada](unidade-listar-processos-visao-detalhada.md)).
A caixa resumida (Recebidos e Gerados da visão R, sem comando próprio) segue
o [golden v2](../../tests/fixtures/caixa-v2.json), conferido por teste unitário:
um **array** de duas listas na ordem recebidos, gerados, cada uma na
[forma única](unidade-recebidos-gerados.md) `{unidade, origem, categoria,
total_declarado, faixa_observada, processos}` (`faixa_observada` é
`{inicio, fim}` ou `null`), sem campo `completa`. Cada processo
publica `id`, `numero` (opaco, sem máscara), `visualizado` e `sigiloso`.
A apresentação não transforma critérios de paginação em propriedades
do domínio.

A unidade e os critérios de paginação publicados na tela precisam ter
identidade inequívoca: em uma mesma resposta, IDs estruturais repetidos
`lnkInfraUnidade`, `hdnTipoVisualizacao`,
`hdnRecebidosHashCriterios` ou `hdnGeradosHashCriterios`
devem ser recusados por erros tipados específicos antes de aceitar
cobertura de páginas ou o rótulo da unidade (refs #57).
Não selecionar o primeiro controle e ocultar o segundo.

Caixa paginada **não** é recusa: o comando devolve a página que a
tela mostra, com total e faixa por tabela (regra em
[páginas da caixa](unidade-listar-processos-paginas.md)). A resposta que
contenha mais de uma ocorrência do mesmo ID estrutural
`tblProcessosRecebidos` ou `tblProcessosGerados` falha com
`TabelasDaCategoriaRepetidas` e categoria tipada: escolher só a
primeira permitiria ocultar processos sem erro.

Não há `ies replay`: as respostas gravadas do Controle passam pelos mesmos
leitores no replay do acervo `visao_do_controle::acervo`
([cassettes](../../docs/cassettes.md#replay-do-acervo)), sem sessão e sem
afirmar snapshot transacional. SOAP e REST precisam de validação própria, sem presunção
de equivalência à tela Web.

## Evidências e verificação

Veja a [matriz da operação](../evidencias/unidade-listar_processos.md)
e os testes Rust ([origem da caixa](../../tests/cli_unidade_origem.rs),
[golden da caixa resumida](../../src/saida/tests_golden.rs)). Nenhuma URL
assinada nem cookie pode aparecer em logs ou no resultado JSON.


## Formulário de login devolvido com HTTP 200 (Refs #57)

Uma requisição GET autenticada à caixa pode receber HTTP 200 e HTML
contendo o formulário de login SIP ou de módulo próprio; sucesso HTTP
**não** equivale a sessão válida. O transporte identifica o formulário
pela estrutura real (form de login e campo de usuário no mesmo form),
retornando `PaginaDeLoginRecebidaNoControle` como falha tipada.
Não classificar tela de assinatura como login; não iniciar nova tentativa,
seguir links, reenviar cookie ou acrescentar requests. Evidência baseada
no fluxo de login documentado pelo Todos, sem validação live nova.

## Legendas únicas por tabela (Refs #57)

Uma tabela de processos recebidos ou gerados com duas ou mais tags
`caption`, inclusive iguais, não oferece interpretação inequívoca do
total/faixa de paginação. O adaptador Web deve falhar com
`LegendasDaTabelaRepetidas` (incluindo categoria) antes de atestar
cobertura das posições ou devolver a caixa. Não escolher simplesmente
a primeira legenda. Um `caption` válido único preserva o comportamento.
Não há mudança de requests ou promessa de snapshot transacional.


## Complemento com páginas HTML privadas: removido (Refs #57)

`--pagina-arquivo` unia ao GET páginas HTML privadas já capturadas, sem
POST. Com a leitura pela visão D e `--pagina N`, a opção foi **removida**
de `listar-processos` (erro de uso, saída 2, sem alias pré-1.0): as telas
da troca de visão são de outra geração que as capturas, e juntá-las não
tem semântica verificável. Cassettes passam pelos leitores no replay do acervo;
`processo NUP --resumo --pagina-arquivo` foi aposentado em 04/11/2026.

## Consulta de apenas uma lista (Refs #57)

`ies unidade recebidos [--json]` e `ies unidade gerados [--json]`
devolvem só a lista pedida, com a mesma regra da caixa: a página
observada, `total_declarado` e `faixa_observada`, sem recusar por
haver outras páginas. A outra tabela continua interpretada e
verificada estruturalmente, mas não aparece na saída. `--saida` exporta
a mesma lista para arquivo privado novo. Seu `--pagina N` pede uma página
de **uma** tabela R e não aceitam `--pagina-arquivo`. Em `listar-processos`,
`--categoria` segue removida (erro de uso, saída 2) e `--pagina N` pede a
página da visão D; ver as [listas Recebidos e Gerados](unidade-recebidos-gerados.md).

## Tipo de resposta HTML conferido sem equivalência por substring (Refs #57)

O GET da caixa e o diagnóstico opt-in do Controle Web só aceitam
`Content-Type` cujo **tipo de mídia principal**, anterior a eventuais
parâmetros separados por `;`, seja exatamente `text/html`
(sem distinguir maiúsculas/minúsculas e ignorando espaços externos).
A presença de `text/html` dentro de um parâmetro de outro tipo,
como `application/json; profile=text/html`, não certifica HTML.
Tipo ausente, `text/plain`, `application/xhtml+xml` ou outros
subtipos são recusados com a variante de transporte
`FalhaTransporte::TipoDeConteudoNaoHtml`. Não há retry nem GET adicional:
o limite, o status e a verificação de login permanecem iguais. A
regra é ensaiada offline; nenhum teste de produção é autorizado.

## Categoria sem identificação da unidade no Controle (Refs #57, #56)

Em `ies unidade recebidos|gerados`, a resposta precisa publicar
a identidade da unidade em `lnkInfraUnidade`, além de demonstrar
cobertura da categoria. Sem esse rótulo, não se deve retornar sucesso
com `unidade: null` nem exportar uma lista de processos que não foi
atribuída a contexto observável. Retornar falha Rust tipada
`FalhaCategoria::UnidadeNaoObservada`, subcódigo
`identificacao_da_unidade_ausente_no_controle`. A ausência na página
não demonstra falta de permissão ou inexistência da unidade.

Sem `--pagina`, permanecem **um GET autorizado, zero POST**, a mesma CLI,
a validação de tabelas/intervalos e a exigência de autorização da
leitura com possíveis efeitos. Essa restrição foi testada offline,
não prova o resultado live em instalações de treinamento.


## Identidade da unidade obrigatória na caixa Web completa (Refs #57, #56)

O comando live `ies unidade listar-processos`, inclusive quando salva
`--saida`, precisa de rótulo inequívoco de unidade publicado pelo Controle
Web (nas telas R e D) antes de devolver processos. Mesmo com total e faixa
cobertos, `unidade: null` **não** comprova a origem dos dados e deve falhar
com `ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle` e o subcódigo
`identificacao_da_unidade_ausente_no_controle`. Não gravar exportação
parcial nem informar sucesso. A regra não requer outro GET, não troca
unidade e não altera a verificação de cobertura das duas categorias.

O replay do acervo (testes `#[ignore]`, não a CLI) lê também capturas sem
identificador de unidade, só para classificar o leitor, **sem alegar
proveniência de leitura live**.
O comportamento é testado com HTML sintético; acesso autenticado nas
instalações de treinamento continua **NOT_RUN**.

## Chaves canônicas de URL e separadores PHP na leitura Web (Refs #57)

Antes de enviar cookie no GET do Controle, o adaptador exige nomes
**literais** de parâmetros ASCII `a-z` e `_` (a regra já rejeitava
aliases, agora com erro tipado); não aceita nomes
normalizados pelo PHP, inclusive `a%63ao`, `infra.hash`, `infra+hash`
ou `acao ajax`. Caso contrário uma chave aparentemente desconhecida
pode sobrescrever `acao` ou `infra_hash` ao ser decodificada pelo
servidor. Recusar separador literal `;` em qualquer valor: servidores
podem tratá-lo como delimitador de query além de `&`.

As variantes Rust tipadas são `ChaveDeConsultaNaoCanonica`
(`chave_de_consulta_do_controle_nao_canonica`) e
`SeparadorDeConsultaAmbiguo`
(`separador_de_consulta_do_controle_ambiguo`).
Mantém parâmetros auxiliares canônicos realmente publicados; não
introduz GET, POST ou fallback. Testes offline não provam compatibilidade
nas instalações UnB/UFRJ/Goiás/PEN (**NOT_RUN**). A gramática precisa
ser revista contra URLs reais autorizadas caso uma instalação use outro
formato; nunca relaxar uma proteção sem evidência.


## Controle de paginação explícito na visão resumida (Refs #57)

Se a página do Controle declarar `hdnTipoVisualizacao=R` e apresentar
controles de paginação da infraestrutura (`lnkInfraProximaPagina*`,
`lnkInfraPaginaAnterior*` ou `selInfraPaginacao*`), **não** considerar
a listagem completa apenas porque os totais da legenda coincidem com
as linhas da página. Recusar com `ErroCaixa::VisaoResumidaComPaginacao`
e subcódigo `visao_resumida_com_paginacao_nao_demonstrada`, sem
exportação ou novas requisições. Os replays e o GET do comando
compartilham a verificação.

A observação do Todos distingue visão resumida (sem paginação comprovada)
e detalhada (`tblProcessosDetalhado`, página em
`hdnDetalhadoPaginaAtual`). **Não** enviar campo de página de outro
formulário, inventar POST, converter a visão detalhada em recebidos/gerados
ou afirmar compatibilidade live sem prova. Os paginadores **por tabela**
(`lnkRecebidos*`, `selGerados*` etc.) são a paginação normal da visão
`R` e seguem a regra das [páginas da caixa](unidade-listar-processos-paginas.md).


## Visão detalhada (Refs #57)

A leitura da D, a troca de visão (sem volta), `--pagina N` e as recusas
estão no [contrato da visão detalhada](unidade-listar-processos-visao-detalhada.md).
Para cada linha aceita, exigir `tr id=P<dígitos>`, um cabeçalho `Processo`
único, link dessa célula para `procedimento_trabalhar` ou
`procedimento_visualizar` e NUP não vazio; legenda única, total, faixa e não
duplicidade, preservando zeros. A página que não cobre o total publica a
`faixa_observada`, sem afirmar lista integral. Não executar links nem JS.

Se uma tela detalhada publicar `hdnDetalhadoPaginaAtual` diferente de `0`
ou controles `lnkInfraProximaPagina*`, `lnkInfraPaginaAnterior*`,
`selInfraPaginacao*`, `lnkDetalhado{Proxima,Anterior,Primeira,Ultima}Pagina*`
ou `selDetalhadoPaginacao*`, sua legenda deve declarar o **intervalo explícito**
(o índice `0` sozinho, UFRJ 4.1.5 numa D de uma página, não é paginação):
coincidência entre total e linhas sem faixa não comprova cobertura e falha
com `faixa_detalhada_sem_intervalo_com_paginacao`.

## Link de processo sigiloso no Controle (Goiás 5.0.3)

Em 02/11/2026, no SEI Goiás 5.0.3 de treinamento (conta pública,
visão resumida), uma linha de `tblProcessosGerados` publicou o link
`procedimento_trabalhar` com a classe `processoVisualizadoSigiloso`;
as demais usavam `processoVisualizado`. O parser só conhecia duas
classes e recusava a caixa inteira com
`LinkDoProcessoNaoIdentificadoNaLinha`. O acervo arquivado de
Goiás (`unidade-listagens-go.json`, `acompanhamento_listar`) já
continha a mesma classe em `tblAcompanhamentos`.

O adaptador `web` reconhece, por **token inteiro** de `class`:

| Classe | `visualizado` | `sigiloso` |
| --- | --- | --- |
| `processoVisualizado` | `true` | `false` |
| `processoNaoVisualizado` | `false` | `false` |
| `processoVisualizadoSigiloso` | `true` | `true` |
| `processoNaoVisualizadoSigiloso` | `false` | `true` |

A última é o par simétrico da nomenclatura e **não foi observada
live**. Outros tokens, como `processoVisitado`, não alteram o estado.
`sigiloso` é fato observado na classe do link, não nível de acesso,
permissão ou propriedade imutável do processo. Um link com duas
classes reconhecidas de estados diferentes falha com
`ClassesDeEstadoDiferentesNoLinkDoProcesso` (visão resumida)
ou `classes_de_estado_diferentes_no_link_detalhado` (detalhada).
Na visão detalhada, `visualizado` e `sigiloso` são `null` quando o
link não traz classe reconhecida.

A importação local aceita exportação anterior sem `sigiloso` como
`false`: aquelas versões recusavam qualquer linha sem as duas classes
não sigilosas. Com 131 recebidos e 7990 gerados em páginas de 10, a
legenda `(N registros - 1 a 10)` passa a ser devolvida como faixa
1–10 de cada tabela; outras páginas só com `recebidos|gerados --pagina N`.
Ensaio offline com HTML sintético; o reensaio live fica **NOT_RUN**.
