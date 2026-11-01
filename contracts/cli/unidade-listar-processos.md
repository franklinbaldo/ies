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

Lê os processos recebidos e gerados da unidade ativa na tela
**Controle de Processos** do SEI Web. A CaixaDeEntrada é da unidade;
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
não segue redirects, não executa login, POST de paginação, nem realiza
escritas ou tentativa de outro backend. Não há configuração persistente
nem automação de autenticação.

`--unidade SIGLA` exige que a página identifique a unidade esperada.
`--saida ARQUIVO` exporta a mesma projeção JSON da caixa **somente depois**
de verificar cobertura de recebidos e gerados. O arquivo novo é criado sem
sobrescrever destinos existentes; em Unix recebe modo privado 0600.
Sem essa opção, a saída de terminal é preservada. O comprovante não
exibe dados dos processos ou caminhos de credenciais. Não implementa
paginação adicional nem declara snapshot transacional.

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

A resposta JSON segue o [golden v2](../../tests/fixtures/caixa-v2.json):
`unidade`, `origem`, `recebidos`, `gerados`; cada agrupamento tem
`processos` e `total_declarado`, sem campo `completa`.
A apresentação não transforma critérios de paginação em propriedades
do domínio.

A unidade e os critérios de paginação publicados na tela precisam ter
identidade inequívoca: em uma mesma resposta, IDs estruturais repetidos
`lnkInfraUnidade`, `hdnTipoVisualizacao`,
`hdnRecebidosHashCriterios` ou `hdnGeradosHashCriterios`
devem ser recusados por erros tipados específicos antes de aceitar
cobertura de páginas ou o rótulo da unidade (refs #57).
Não selecionar o primeiro controle e ocultar o segundo.

O comando de domínio só retorna sucesso quando as tabelas disponíveis
cobrem os totais declarados. A resposta que contenha mais de uma ocorrência
do mesmo ID estrutural `tblProcessosRecebidos` ou `tblProcessosGerados`
deve falhar com `tabelas_da_categoria_repetidas` e categoria tipada:
escolher só a primeira permitiria ocultar processos sem erro. Se a tela exige outras páginas, ele
retorna `paginas_do_controle_nao_obtidas` e **não publica um resultado
parcial como se fosse toda a caixa**. Este recorte não implementa o
POST de paginação; portanto, algumas unidades ainda não podem ser
consultadas integralmente.

O replay de [evidências offline](../../docs/replay.md) continua em
`ies replay CENARIO`, independente de sessão e sem afirmar snapshot
transacional. SOAP e REST precisam de validação própria, sem presunção
de equivalência à tela Web.

## Evidências e verificação

Veja a [matriz da operação](../evidencias/unidade-listar_processos.md)
e os [testes Rust](../../tests/replay_unidade.rs). Nenhuma URL
assinada nem cookie pode aparecer em logs ou no resultado JSON.


## Formulário de login devolvido com HTTP 200 (Refs #57)

Uma requisição GET autenticada à caixa pode receber HTTP 200 e HTML
contendo o formulário de login SIP ou de módulo próprio; sucesso HTTP
**não** equivale a sessão válida. O transporte identifica o formulário
pela estrutura real (form de login e campo de usuário no mesmo form),
retornando `pagina_de_login_recebida_no_controle` como falha tipada.
Não classificar tela de assinatura como login; não iniciar nova tentativa,
seguir links, reenviar cookie ou acrescentar requests. Evidência baseada
no fluxo de login documentado pelo Todos, sem validação live nova.

## Legendas únicas por tabela (Refs #57)

Uma tabela de processos recebidos ou gerados com duas ou mais tags
`caption`, inclusive iguais, não oferece interpretação inequívoca do
total/faixa de paginação. O adaptador Web deve falhar com
`legendas_da_tabela_repetidas` (incluindo categoria) antes de atestar
cobertura das posições ou devolver a caixa. Não escolher simplesmente
a primeira legenda. Um `caption` válido único preserva o comportamento.
Não há mudança de requests ou promessa de snapshot transacional.


## Complemento com páginas HTML privadas previamente capturadas (Refs #57)

A opção aditiva `ies unidade listar-processos --pagina-arquivo ARQUIVO`
pode ser repetida para acrescentar **páginas HTML já capturadas**
ao GET autorizado da tela inicial. O IES não produz nem repete
POST de paginação: orçamento de rede permanece **um GET**.
Cada arquivo precisa ser regular, privado (0600 no Unix), não pode
ser link simbólico final no Linux, tem limite de 8 MB e deve
conter HTML UTF-8 previamente normalizado. Recusam-se caminhos
duplicados, mais de 199 complementos ou mais de 48 MB de HTML
total. A CLI não publica HTML nem URLs assinadas ou cookies.

A união utiliza as verificações existentes de
`paginas::juntar`: rótulo de unidade, visualização, hashes de
critérios, faixas contíguas, IDs e totais de ambas as tabelas.
Um complemento ausente, divergente ou insuficiente deve produzir
erro e **não** devolver caixa parcial. Quando complementos são
usados, a saída JSON declara `origem: web_com_paginas_privadas`,
distinguindo da consulta baseada apenas no GET atual.

As páginas extras precisam ter sido capturadas legitimamente
pela própria pessoa autorizada na **mesma sessão, unidade e
configuração**; os hashes de critério são apenas marcadores
opacos, **não** provas criptográficas de origem, atualidade
ou snapshot do SEI. O IES não executa coleta autenticada, não
pode provar a frescura dos arquivos e não afirma suporte live
à paginação POST. O modo sem a opção e o golden JSON existente
permanecem inalterados.

## Consulta de apenas uma categoria verificada (Refs #57)

`ies unidade listar-processos --categoria recebidos|gerados [--json]`
é uma opção **aditiva** da consulta Web existente: permite devolver
apenas a categoria pedida quando suas posições e total declarado
estão inteiramente cobertos no GET inicial, **mesmo que a outra
categoria exija páginas adicionais**. O orçamento permanece **um
GET**, sem POST de paginação, mudança de unidade ou autenticação
nova. A outra categoria continua sendo interpretada e verificada
estruturalmente, mas não aparece na saída projetada.

A projeção JSON opt-in é
`{unidade, origem: "web", categoria, total_declarado, processos}`,
sem campos de completude no domínio ou promessa de snapshot.
`--saida` exporta essa mesma projeção para arquivo privado novo,
sem sobrescrever nem expor processos no comprovante.
Se a categoria **selecionada** não estiver integralmente coberta,
falhar com `paginas_recebidos_nao_obtidas` ou
`paginas_gerados_nao_obtidas` (erros Rust tipados), sem publicar
lista parcial. Com `--categoria` ausente, contrato e golden de
todas as categorias seguem inalterados.

Exemplo empírico: numa instalação onde Gerados já tem todas as
91 linhas declaradas mas Recebidos exibe 100 de 168, a opção
`--categoria gerados` pode relatar somente Gerados sem afirmar
a cobertura de Recebidos. Não se infere comportamento idêntico
nas outras instalações; validar em treinamento autorizado antes
de declarar operação live multiplataforma.

A opção `--categoria` é incompatível com `--pagina-arquivo`: a seleção
considera exclusivamente a primeira página do GET e não ignora páginas
privadas indicadas pelo operador.

## Tipo de resposta HTML conferido sem equivalência por substring (Refs #57)

O GET da caixa e o diagnóstico opt-in do Controle Web só aceitam
`Content-Type` cujo **tipo de mídia principal**, anterior a eventuais
parâmetros separados por `;`, seja exatamente `text/html`
(sem distinguir maiúsculas/minúsculas e ignorando espaços externos).
A presença de `text/html` dentro de um parâmetro de outro tipo,
como `application/json; profile=text/html`, não certifica HTML.
Tipo ausente, `text/plain`, `application/xhtml+xml` ou outros
subtipos continuam recusados com a variante de transporte existente
`FalhaTransporte::RespostaInesperada`. Não há retry nem GET adicional:
o limite, o status e a verificação de login permanecem iguais. A
regra é ensaiada offline; nenhum teste de produção é autorizado.

## Categoria sem identificação da unidade no Controle (Refs #57, #56)

No recorte `--categoria recebidos|gerados`, a resposta precisa publicar
a identidade da unidade em `lnkInfraUnidade`, além de demonstrar
cobertura da categoria. Sem esse rótulo, não se deve retornar sucesso
com `unidade: null` nem exportar uma lista de processos que não foi
atribuída a contexto observável. Retornar falha Rust tipada
`FalhaCategoria::UnidadeNaoObservada`, subcódigo
`identificacao_da_unidade_ausente_no_controle`. A ausência na página
não demonstra falta de permissão ou inexistência da unidade.

Permanecem os mesmos **um GET autorizado, zero POST**, a mesma CLI,
a validação de tabelas/intervalos e a exigência de autorização da
leitura com possíveis efeitos. Essa restrição foi testada offline,
não prova o resultado live em instalações de treinamento.


## Identidade da unidade obrigatória na caixa Web completa (Refs #57, #56)

O comando live `ies unidade listar-processos`, inclusive quando salva
`--saida`, precisa de rótulo inequívoco de unidade publicado pelo Controle
Web antes de devolver processos. Mesmo que ambos os totais e faixas estejam
cobertos, `unidade: null` **não** comprova a origem dos dados e deve falhar
com `ErroCaixa::IdentificacaoDaUnidadeAusenteNoControle` e o subcódigo
`identificacao_da_unidade_ausente_no_controle`. Não gravar exportação
parcial nem informar sucesso. A regra não requer outro GET, não troca
unidade e não altera a verificação de cobertura das duas categorias.

`ies replay` permanece um instrumento de análise de cassettes antigos:
se a captura não possui identificador de unidade, pode apresentar o conteúdo
somente com escopo de replay, **sem alegar proveniência de leitura live**.
O comportamento é testado com HTML sintético; acesso autenticado nas
instalações de treinamento continua **NOT_RUN**.
